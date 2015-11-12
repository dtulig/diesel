use schema::*;
use yaqb::*;

#[test]
fn filter_by_int_equality() {
    use schema::users::dsl::*;

    let connection = connection_with_sean_and_tess_in_users_table();

    let sean = User::new(1, "Sean");
    let tess = User::new(2, "Tess");
    assert_eq!(Some(sean), connection.query_one(users.filter(id.eq(1))).unwrap());
    assert_eq!(Some(tess), connection.query_one(users.filter(id.eq(2))).unwrap());
    assert_eq!(None::<User>, connection.query_one(users.filter(id.eq(3))).unwrap());
}

#[test]
fn filter_by_string_equality() {
    use schema::users::dsl::*;

    let connection = connection_with_sean_and_tess_in_users_table();

    let sean = User::new(1, "Sean");
    let tess = User::new(2, "Tess");
    assert_eq!(Some(sean), connection.query_one(users.filter(name.eq("Sean"))).unwrap());
    assert_eq!(Some(tess), connection.query_one(users.filter(name.eq("Tess"))).unwrap());
    assert_eq!(None::<User>, connection.query_one(users.filter(name.eq("Jim"))).unwrap());
}

#[test]
fn filter_by_equality_on_nullable_columns() {
    use schema::users::dsl::*;

    let connection = connection();
    setup_users_table(&connection);
    let data = vec![
        NewUser::new("Sean", Some("black")),
        NewUser::new("Tess", Some("brown")),
        NewUser::new("Jim", Some("black")),
    ];
    connection.insert_without_return(&users, &data).unwrap();

    let sean = User::with_hair_color(1, "Sean", "black");
    let tess = User::with_hair_color(2, "Tess", "brown");
    let jim = User::with_hair_color(3, "Jim", "black");
    let source = users.filter(hair_color.eq("black"));
    assert_eq!(vec![sean, jim], connection.query_all(source).unwrap().collect::<Vec<_>>());
    let source = users.filter(hair_color.eq("brown"));
    assert_eq!(vec![tess], connection.query_all(source).unwrap().collect::<Vec<_>>());
}

#[test]
fn filter_after_joining() {
    use schema::users::name;

    let connection = connection_with_sean_and_tess_in_users_table();
    setup_posts_table(&connection);
    connection.execute("INSERT INTO POSTS (title, user_id) VALUES ('Hello', 1), ('World', 2)")
        .unwrap();

    let sean = User::new(1, "Sean");
    let tess = User::new(2, "Tess");
    let seans_post = Post::new(1, 1, "Hello", None);
    let tess_post = Post::new(2, 2, "World", None);
    let source = users::table.inner_join(posts::table);
    assert_eq!(Some((sean, seans_post)),
        connection.query_one(source.filter(name.eq("Sean"))).unwrap());
    assert_eq!(Some((tess, tess_post)),
        connection.query_one(source.filter(name.eq("Tess"))).unwrap());
    assert_eq!(None::<(User, Post)>,
        connection.query_one(source.filter(name.eq("Jim"))).unwrap());
}

#[test]
fn select_then_filter() {
    use schema::users::dsl::*;

    let connection = connection_with_sean_and_tess_in_users_table();

    let source = users.select(name);
    assert_eq!(Some("Sean".to_string()),
        connection.query_one(source.filter(name.eq("Sean"))).unwrap());
    assert_eq!(Some("Tess".to_string()),
        connection.query_one(source.filter(name.eq("Tess"))).unwrap());
    assert_eq!(None::<String>, connection.query_one(source.filter(name.eq("Jim"))).unwrap());
}

#[test]
fn filter_then_select() {
    use schema::users::dsl::*;

    let connection = connection();
    setup_users_table(&connection);
    let data = [NewUser::new("Sean", None), NewUser::new("Tess", None)];
    connection.insert_without_return(&users, &data).unwrap();

    assert_eq!(Some("Sean".to_string()),
        connection.query_one(users.filter(name.eq("Sean")).select(name)).unwrap());
    assert_eq!(Some("Tess".to_string()),
        connection.query_one(users.filter(name.eq("Tess")).select(name)).unwrap());
    assert_eq!(None::<String>, connection.query_one(users.filter(name.eq("Jim")).select(name)).unwrap());
}

#[test]
fn filter_on_multiple_columns() {
    use schema::users::dsl::*;

    let connection = connection();
    setup_users_table(&connection);
    let data = [
        NewUser::new("Sean", Some("black")),
        NewUser::new("Sean", Some("brown")),
        NewUser::new("Sean", None),
        NewUser::new("Tess", Some("black")),
        NewUser::new("Tess", Some("brown")),
    ];
    assert_eq!(5, connection.insert_without_return(&users, &data).unwrap());

    let black_haired_sean = User::with_hair_color(1, "Sean", "black");
    let brown_haired_sean = User::with_hair_color(2, "Sean", "brown");
    let _bald_sean = User::new(3, "Sean");
    let black_haired_tess = User::with_hair_color(4, "Tess", "black");
    let brown_haired_tess = User::with_hair_color(5, "Tess", "brown");

    let source = users.filter(name.eq("Sean").and(hair_color.eq("black")));
    assert_eq!(vec![black_haired_sean], connection.query_all(source).unwrap()
        .collect::<Vec<_>>());

    let source = users.filter(name.eq("Sean").and(hair_color.eq("brown")));
    assert_eq!(vec![brown_haired_sean], connection.query_all(source).unwrap()
        .collect::<Vec<_>>());

    let source = users.filter(name.eq("Tess").and(hair_color.eq("black")));
    assert_eq!(vec![black_haired_tess], connection.query_all(source).unwrap()
        .collect::<Vec<_>>());

    let source = users.filter(name.eq("Tess").and(hair_color.eq("brown")));
    assert_eq!(vec![brown_haired_tess], connection.query_all(source).unwrap()
        .collect::<Vec<_>>());
}

#[test]
fn filter_called_twice_means_same_thing_as_and() {
    use schema::users::dsl::*;

    let connection = connection();
    setup_users_table(&connection);
    let data = [
        NewUser::new("Sean", Some("black")),
        NewUser::new("Sean", Some("brown")),
        NewUser::new("Sean", None),
        NewUser::new("Tess", Some("black")),
        NewUser::new("Tess", Some("brown")),
    ];
    assert_eq!(5, connection.insert_without_return(&users, &data).unwrap());

    let black_haired_sean = User::with_hair_color(1, "Sean", "black");
    let brown_haired_sean = User::with_hair_color(2, "Sean", "brown");
    let _bald_sean = User::new(3, "Sean");
    let black_haired_tess = User::with_hair_color(4, "Tess", "black");
    let brown_haired_tess = User::with_hair_color(5, "Tess", "brown");

    let source = users.filter(name.eq("Sean")).filter(hair_color.eq("black"));
    assert_eq!(vec![black_haired_sean], connection.query_all(source).unwrap()
        .collect::<Vec<_>>());

    let source = users.filter(name.eq("Sean")).filter(hair_color.eq("brown"));
    assert_eq!(vec![brown_haired_sean], connection.query_all(source).unwrap()
        .collect::<Vec<_>>());

    let source = users.filter(name.eq("Tess")).filter(hair_color.eq("black"));
    assert_eq!(vec![black_haired_tess], connection.query_all(source).unwrap()
        .collect::<Vec<_>>());

    let source = users.filter(name.eq("Tess")).filter(hair_color.eq("brown"));
    assert_eq!(vec![brown_haired_tess], connection.query_all(source).unwrap()
        .collect::<Vec<_>>());
}

table! {
    points (x) {
        x -> Integer,
        y -> Integer,
    }
}

#[test]
fn filter_on_column_equality() {
    use self::points::dsl::*;

    let connection = connection();
    connection.execute("CREATE TABLE points (x INTEGER NOT NULL, y INTEGER NOT NULL)").unwrap();
    connection.execute("INSERT INTO POINTS (x, y) VALUES (1, 1), (1, 2), (2, 2)").unwrap();

    let expected_data = vec![(1, 1), (2, 2)];
    let query = points.filter(x.eq(y));
    let data: Vec<_> = connection.query_all(query).unwrap().collect();
    assert_eq!(expected_data, data);
}