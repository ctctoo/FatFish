use project_hub::database;
use project_hub::error::AppError;
use project_hub::models::collection::CollectionInput;
use project_hub::models::link::LinkInput;
use project_hub::models::project::{ProjectFilter, ProjectInput};
use project_hub::models::tag::TagInput;
use project_hub::models::todo::TodoInput;
use project_hub::repository::{activity, collection, git, link, project, tag, todo};
use project_hub::models::git::GitInfo;
use rusqlite::Connection;

fn test_conn() -> Connection {
    let mut conn = Connection::open_in_memory().expect("in-memory db");
    database::migrate(&mut conn).expect("migrate");
    conn
}

fn input(name: &str, path: &str) -> ProjectInput {
    ProjectInput {
        name: name.into(),
        path: path.into(),
        description: None,
        status: None,
        cover_emoji: None,
        cover_color: None,
        notes: None,
    }
}

#[test]
fn project_crud_roundtrip() {
    let conn = test_conn();

    let p = project::insert(&conn, &input("  Thesis  ", r"D:\Research\Thesis\")).expect("insert");
    // 路径去首尾空白与尾部斜杠
    assert_eq!(p.path, r"D:\Research\Thesis");
    assert_eq!(p.name, "Thesis");
    assert_eq!(p.status, "IN_PROGRESS");
    assert!(!p.favorite);

    // 大小写不敏感的重复路径检测
    assert!(project::exists_with_path(&conn, r"d:\RESEARCH\THESIS").unwrap());

    project::set_favorite(&conn, &p.id, true).unwrap();
    project::mark_opened(&conn, &p.id).unwrap();

    let updated = project::update(
        &conn,
        &p.id,
        &ProjectInput {
            name: "Thesis 2".into(),
            path: r"D:\Research\Thesis2".into(),
            description: Some("desc".into()),
            status: Some("PLANNED".into()),
            cover_emoji: Some("📘".into()),
            cover_color: Some("#123456".into()),
            notes: Some("notes".into()),
        },
    )
    .expect("update");
    assert_eq!(updated.name, "Thesis 2");
    assert_eq!(updated.status, "PLANNED");
    assert_eq!(updated.description.as_deref(), Some("desc"));
    assert!(updated.favorite);

    let fetched = project::get(&conn, &p.id).unwrap().expect("get");
    assert_eq!(fetched.last_opened_at, updated.last_opened_at);
    assert!(!fetched.last_opened_at.is_empty());

    let del = project::delete(&conn, &p.id).unwrap();
    assert_eq!(del, 1);
    assert!(project::get(&conn, &p.id).unwrap().is_none());

    // 删除不存在的记录时 update 应返回 NotFound
    let err = project::update(&conn, "missing", &input("x", "y")).unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[test]
fn project_list_filters_combine() {
    let conn = test_conn();

    let alpha = project::insert(&conn, &input("Alpha", "C:\\a")).unwrap();
    let beta = project::insert(
        &conn,
        &ProjectInput {
            name: "Beta".into(),
            path: "C:\\b".into(),
            description: Some("research notes".into()),
            status: Some("COMPLETED".into()),
            cover_emoji: None,
            cover_color: None,
            notes: None,
        },
    )
    .unwrap();

    // 标签 / 集合 / 链接挂到 Beta 上，用于关键词联查
    let tag = tag::insert(&conn, &TagInput { name: "Rust".into(), color: Some("#ff0000".into()) }).unwrap();
    tag::set_project_tags(&conn, &beta.id, std::slice::from_ref(&tag.id)).unwrap();
    let coll = collection::insert(&conn, &CollectionInput { name: "Work".into() }).unwrap();
    collection::set_project_collections(&conn, &beta.id, std::slice::from_ref(&coll.id)).unwrap();
    link::insert(
        &conn,
        &beta.id,
        &LinkInput { title: "Docs".into(), url: "https://example.com".into(), link_type: Some("docs".into()) },
    )
    .unwrap();

    // 关键词命中标签名
    let hits = project::list(&conn, &ProjectFilter { query: Some("rust".into()), ..Default::default() }).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, beta.id);
    assert_eq!(hits[0].tags.len(), 1);
    assert_eq!(hits[0].collections.len(), 1);
    assert_eq!(hits[0].links.len(), 1);

    // 关键词命中链接标题
    let hits = project::list(&conn, &ProjectFilter { query: Some("docs".into()), ..Default::default() }).unwrap();
    assert_eq!(hits.len(), 1);

    // 状态筛选
    let hits = project::list(&conn, &ProjectFilter { status: Some("COMPLETED".into()), ..Default::default() }).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, beta.id);

    // 标签 + 状态组合
    let hits = project::list(
        &conn,
        &ProjectFilter {
            status: Some("COMPLETED".into()),
            tag_id: Some(tag.id.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(hits.len(), 1);

    // 集合筛选
    let hits = project::list(&conn, &ProjectFilter { collection_id: Some(coll.id), ..Default::default() }).unwrap();
    assert_eq!(hits.len(), 1);

    // 名称排序
    let hits = project::list(&conn, &ProjectFilter { sort: Some("name".into()), ..Default::default() }).unwrap();
    assert_eq!(hits[0].name, "Alpha");
    assert_eq!(hits[1].name, "Beta");

    // 最近 8 条
    let hits = project::list(&conn, &ProjectFilter { recent: Some(true), ..Default::default() }).unwrap();
    assert!(hits.len() <= 8);
    let _ = alpha;
}

#[test]
fn tag_collection_link_crud() {
    let conn = test_conn();
    let project = project::insert(&conn, &input("P", "C:\\p")).unwrap();

    // Tag
    let t1 = tag::insert(&conn, &TagInput { name: " Research ".into(), color: None }).unwrap();
    assert_eq!(t1.name, "Research");
    let t2 = tag::insert(&conn, &TagInput { name: "Urgent".into(), color: Some("#f00".into()) }).unwrap();
    tag::set_project_tags(&conn, &project.id, &[t1.id.clone(), t2.id.clone()]).unwrap();
    let tags = tag::list_for_project(&conn, &project.id).unwrap();
    assert_eq!(tags.len(), 2);
    // 全量替换
    tag::set_project_tags(&conn, &project.id, std::slice::from_ref(&t2.id)).unwrap();
    let tags = tag::list_for_project(&conn, &project.id).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].id, t2.id);
    tag::update(&conn, &t1.id, &TagInput { name: "Renamed".into(), color: None }).unwrap();
    tag::delete(&conn, &t1.id).unwrap();
    assert!(matches!(tag::update(&conn, &t1.id, &TagInput { name: "x".into(), color: None }), Err(AppError::NotFound(_))));

    // Collection
    let c1 = collection::insert(&conn, &CollectionInput { name: "Work".into() }).unwrap();
    collection::set_project_collections(&conn, &project.id, std::slice::from_ref(&c1.id)).unwrap();
    assert_eq!(collection::list_for_project(&conn, &project.id).unwrap().len(), 1);
    assert_eq!(collection::list(&conn).unwrap().len(), 1);
    collection::update(&conn, &c1.id, &CollectionInput { name: "Side".into() }).unwrap();
    collection::delete(&conn, &c1.id).unwrap();

    // Link
    let l1 = link::insert(&conn, &project.id, &LinkInput { title: " GitHub ".into(), url: " https://github.com/x ".into(), link_type: Some("github".into()) }).unwrap();
    assert_eq!(l1.title, "GitHub");
    assert_eq!(l1.url, "https://github.com/x");
    let l2 = link::insert(&conn, &project.id, &LinkInput { title: "Site".into(), url: "https://x.dev".into(), link_type: None }).unwrap();
    let links = link::list_for_project(&conn, &project.id).unwrap();
    assert_eq!(links.len(), 2);
    // 保持插入顺序
    assert_eq!(links[0].id, l1.id);
    assert_eq!(links[1].id, l2.id);
    link::update(&conn, &l2.id, &LinkInput { title: "Site2".into(), url: "https://y.dev".into(), link_type: Some("website".into()) }).unwrap();
    link::delete(&conn, &l1.id).unwrap();
    assert_eq!(link::list_for_project(&conn, &project.id).unwrap().len(), 1);
}

#[test]
fn todo_crud_and_order() {
    let conn = test_conn();
    let project = project::insert(&conn, &input("P", "C:\\p")).unwrap();

    let a = todo::insert(&conn, &TodoInput { title: "First".into(), project_id: Some(project.id.clone()), due_date: None }).unwrap();
    let b = todo::insert(&conn, &TodoInput { title: "Second".into(), project_id: Some(project.id), due_date: Some("2026-09-01".into()) }).unwrap();

    let todos = todo::list(&conn).unwrap();
    assert_eq!(todos.len(), 2);
    // 新任务排前面（sort_order 递减）
    assert_eq!(todos[0].id, b.id);
    assert_eq!(todos[0].project_name.as_deref(), Some("P"));

    todo::set_done(&conn, &b.id, true).unwrap();
    let todos = todo::list(&conn).unwrap();
    // 未完成在前
    assert_eq!(todos[0].id, a.id);
    assert!(todos[1].done);

    todo::update(&conn, &a.id, &TodoInput { title: "Renamed".into(), project_id: None, due_date: None }).unwrap();
    assert_eq!(todo::list(&conn).unwrap()[0].title, "Renamed");
    todo::delete(&conn, &a.id).unwrap();
    assert_eq!(todo::list(&conn).unwrap().len(), 1);
}

#[test]
fn git_info_upsert_and_activity() {
    let conn = test_conn();
    let project = project::insert(&conn, &input("P", "C:\\p")).unwrap();

    let info = GitInfo {
        remote_url: Some("https://github.com/x/p".into()),
        branch: Some("main".into()),
        commit_hash: Some("abc123".into()),
        commit_message: Some("init".into()),
        commit_time: Some("2026-08-01".into()),
        is_dirty: Some(true),
    };
    git::upsert(&conn, &project.id, &info).unwrap();
    let loaded = project::get(&conn, &project.id).unwrap().unwrap();
    assert_eq!(loaded.git_info.as_ref().unwrap().branch.as_deref(), Some("main"));
    assert_eq!(loaded.git_info.as_ref().unwrap().is_dirty, Some(true));

    // 更新合并
    git::upsert(&conn, &project.id, &GitInfo { branch: Some("dev".into()), ..Default::default() }).unwrap();
    let loaded = project::get(&conn, &project.id).unwrap().unwrap();
    let gi = loaded.git_info.unwrap();
    assert_eq!(gi.branch.as_deref(), Some("dev"));
    // upsert 是整行覆盖，未提供的字段变 None
    assert_eq!(gi.remote_url, None);

    activity::log(&conn, &project.id, "created", "项目创建");
    activity::log(&conn, &project.id, "updated", "更新备注");
    let acts = activity::list_for_project(&conn, &project.id, 10).unwrap();
    assert_eq!(acts.len(), 2);
    assert_eq!(acts[0].kind, "updated");
}
