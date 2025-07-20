use postgres::{Client, NoTls, Error};

pub fn insert_user(client: &mut Client, name: &str) -> Result<i32, Error> {
    let insert = client.query_opt("INSERT INTO users(name) VALUES ($1) ON CONFLICT (name) DO NOTHING RETURNING id", &[&name])?;

    if let Some(row) = insert {
        Ok(row.get("id"))
    } else {
        let select = client.query_one(
            "SELECT id FROM users WHERE name = $1", 
            &[&name]
        )?;
        Ok(select.get("id"))
    }
}

pub fn get_user_by_id(client: &mut Client, id: i32) -> Result<Option<String>, Error> {
    let row = client.query_opt("SELECT name FROM users WHERE id = $1", &[&id])?;
    Ok(row.map(|r| r.get("name")))
}

pub fn get_user_by_name(client: &mut Client, name: &str) -> Result<Option<i32>, Error> {
    let row = client.query_opt("SELECT id FROM users WHERE name = $1", &[&name])?;
    Ok(row.map(|r| r.get("id")))
}

pub fn update_user_name(client: &mut Client, id: i32, new_name: &str) -> Result<Option<i32>, Error> {
    let row = client.query_opt("UPDATE users SET name = $1 WHERE id = $2", &[&new_name, &id])?;
    Ok(row.map(|r| r.get("id")))
}

pub fn delete_user(client: &mut Client, id: i32) -> Result<u64, Error> {
    let count = client.execute("DELETE FROM users WHERE id = $1", &[&id])?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn connect_test_client() -> Client {
        let db_url = env::var("DATABASE_URL").unwrap();
        Client::connect(&db_url, NoTls).unwrap()
    }

    #[test]
    fn test_create_user() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let new_user = "testuser";

        let id = insert_user(&mut client, &new_user).unwrap();
        let fetched_name = get_user_by_id(&mut client, id).unwrap();
        assert_eq!(fetched_name, Some(new_user.to_string()));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_get_user_by_id() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let new_user = "testuser";

        let id = insert_user(&mut client, &new_user).unwrap();
        let row = get_user_by_id(&mut client, id).unwrap();
        assert_eq!(row, Some(new_user.to_string()));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_get_user_by_name() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let new_user = "testuser";

        let id = insert_user(&mut client, &new_user).unwrap();
        let fetched = get_user_by_name(&mut client, &new_user).unwrap();
        assert_eq!(fetched, Some(id));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_update_user_name() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let old_name = "testuser".to_string();
        let new_name = "Updated".to_string();

        let id = insert_user(&mut client, &old_name).unwrap();
        update_user_name(&mut client, id, &new_name).unwrap();

        let updated_id = get_user_by_name(&mut client, &new_name).unwrap();
        assert_eq!(updated_id, Some(id));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_delete_user() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let name = "testuser".to_string();
        let id = insert_user(&mut client, &name).unwrap();
        let count = delete_user(&mut client, id).unwrap();
        assert_eq!(count, 1);

        client.batch_execute("ROLLBACK").unwrap();
    }
}
