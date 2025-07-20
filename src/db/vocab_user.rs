use postgres::{Client, NoTls, Error};

pub fn insert_vocab_user(client: &mut Client, id_user: i32, id_word: i32) -> Result<u64, Error> {
    let count = client.execute(
        "INSERT INTO vocabulary_user (id_user, id_word) VALUES ($1, $2) ON CONFLICT (id_user, id_word) DO NOTHING",
        &[&id_user, &id_word]
    )?;
    Ok(count)
}

pub fn insert_vocab_users_multiple(client: &mut Client, id_user: i32, id_words: &[i32]) -> Result<u64, Error> {
    if id_words.is_empty() {
        return Ok(0);
    }

    let mut query = String::from("INSERT INTO vocabulary_user (id_user, id_word) VALUES ");
    let mut params: Vec<&(dyn postgres::types::ToSql + Sync)> = Vec::new();
    let mut placeholders = Vec::new();

    for (i, id_word) in id_words.iter().enumerate() {
        placeholders.push(format!("(${}, ${})", 2 * i + 1, 2 * i + 2));
        params.push(&id_user as &(dyn postgres::types::ToSql + Sync));
        params.push(id_word as &(dyn postgres::types::ToSql + Sync));
    }

    query.push_str(&placeholders.join(", "));
    query.push_str(" ON CONFLICT (id_user, id_word) DO NOTHING");

    let count = client.execute(&query, &params)?;
    Ok(count)
}

pub fn get_words_for_user(client: &mut Client, id_user: i32) -> Result<Vec<i32>, Error> {
    let rows = client.query("SELECT id_word FROM vocabulary_user WHERE id_user = $1", &[&id_user])?;
    let word_ids: Vec<i32> = rows.iter().map(|row| row.get("id_word")).collect();
    Ok(word_ids)
}

pub fn get_users_for_word(client: &mut Client, id_word: i32) -> Result<Vec<i32>, Error> {
    let rows = client.query("SELECT id_user FROM vocabulary_user WHERE id_word = $1", &[&id_word])?;
    let user_ids: Vec<i32> = rows.iter().map(|row| row.get("id_user")).collect();
    Ok(user_ids)
}

pub fn delete_vocab_user(client: &mut Client, id_user: i32, id_word: i32) -> Result<u64, Error> {
    let count = client.execute(
        "DELETE FROM vocabulary_user WHERE id_user = $1 AND id_word = $2",
        &[&id_user, &id_word]
    )?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use crate::db::users::insert_user;
    use crate::db::words::insert_word;

    fn connect_test_client() -> Client {
        let db_url = env::var("DATABASE_URL").unwrap();
        Client::connect(&db_url, NoTls).unwrap()
    }

    #[test]
    fn test_insert_vocab_user() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let user_id = insert_user(&mut client, "testuser").unwrap();
        let word_id = insert_word(&mut client, "testword", "english").unwrap();

        let count = insert_vocab_user(&mut client, user_id, word_id).unwrap();
        assert_eq!(count, 1);

        let count2 = insert_vocab_user(&mut client, user_id, word_id).unwrap();
        assert_eq!(count2, 0);

        client.batch_execute("ROLLBACK").unwrap();
    }
    
    #[test]
    fn test_insert_vocab_users_multiple() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let user_id = insert_user(&mut client, "testuser").unwrap();
        let word_id1 = insert_word(&mut client, "word1", "english").unwrap();
        let word_id2 = insert_word(&mut client, "word2", "english").unwrap();
        let word_ids = vec![word_id1, word_id2];

        let count = insert_vocab_users_multiple(&mut client, user_id, &word_ids).unwrap();
        assert_eq!(count, 2);

        let count2 = insert_vocab_users_multiple(&mut client, user_id, &word_ids).unwrap();
        assert_eq!(count2, 0);

        let words = get_words_for_user(&mut client, user_id).unwrap();
        assert_eq!(words.len(), 2);
        assert!(words.contains(&word_id1));
        assert!(words.contains(&word_id2));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_get_words_for_user() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let user_id = insert_user(&mut client, "testuser").unwrap();
        let word_id1 = insert_word(&mut client, "word1", "english").unwrap();
        let word_id2 = insert_word(&mut client, "word2", "english").unwrap();

        insert_vocab_user(&mut client, user_id, word_id1).unwrap();
        insert_vocab_user(&mut client, user_id, word_id2).unwrap();

        let words = get_words_for_user(&mut client, user_id).unwrap();
        assert_eq!(words.len(), 2);
        assert!(words.contains(&word_id1));
        assert!(words.contains(&word_id2));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_get_users_for_word() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let user_id1 = insert_user(&mut client, "user1").unwrap();
        let user_id2 = insert_user(&mut client, "user2").unwrap();
        let word_id = insert_word(&mut client, "testword", "english").unwrap();

        insert_vocab_user(&mut client, user_id1, word_id).unwrap();
        insert_vocab_user(&mut client, user_id2, word_id).unwrap();

        let users = get_users_for_word(&mut client, word_id).unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.contains(&user_id1));
        assert!(users.contains(&user_id2));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_delete_vocab_user() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let user_id = insert_user(&mut client, "testuser").unwrap();
        let word_id = insert_word(&mut client, "testword", "english").unwrap();

        insert_vocab_user(&mut client, user_id, word_id).unwrap();
        let count = delete_vocab_user(&mut client, user_id, word_id).unwrap();
        assert_eq!(count, 1);

        let words = get_words_for_user(&mut client, user_id).unwrap();
        assert_eq!(words.len(), 0);

        client.batch_execute("ROLLBACK").unwrap();
    }
}
