use postgres::{Client, NoTls, Error};
use std::collections::HashSet;

pub fn insert_word(client: &mut Client, text: &str, language: &str) -> Result<i32, Error> {
    let insert = client.query_opt("INSERT INTO words(text, language) VALUES ($1, $2)
                                ON CONFLICT (text, language) DO NOTHING RETURNING id", &[&text, &language])?;
    
    if let Some(row) = insert {
        Ok(row.get("id"))
    } else {
        let select = client.query_one(
            "SELECT id FROM words WHERE text = $1 AND language = $2",
            &[&text, &language]
        )?;
        Ok(select.get("id"))
    }
}

pub fn insert_multiple_words(client: &mut Client, words: &[String], language: &str) -> Result<Vec<i32>, Error> {
    let existing_words = get_existing_words(client, words, language)?;

    let new_words: Vec<&String> = words.iter().filter(|w| !existing_words.contains(w.as_str())).collect();

    if new_words.is_empty() {
        return Ok(Vec::new());
    }

    let texts: Vec<&str> = new_words.iter().map(|s| s.as_str()).collect();
    let languages: Vec<&str> = vec![language; new_words.len()];

    let query = "INSERT INTO words (text, language)
                 SELECT * FROM UNNEST($1::text[], $2::text[])
                 ON CONFLICT (text, language) DO NOTHING
                 RETURNING id";
    let rows = client.query(query, &[&texts, &languages])?;

    let inserted_ids: Vec<i32> = rows.iter().map(|row| row.get("id")).collect();

    Ok(inserted_ids)
}

fn get_existing_words(client: &mut Client, words: &[String], language: &str) -> Result<HashSet<String>, Error> {
    let texts: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
    let query = "SELECT text FROM words WHERE text = ANY($1) AND language = $2";
    let rows = client.query(query, &[&texts, &language])?;

    let mut existing = HashSet::new();
    for row in rows {
        let text: String = row.get("text");
        existing.insert(text);
    }

    Ok(existing)
}

pub fn get_id_word(client: &mut Client, text: &str, language: &str) -> Result<Option<i32>, Error> {
    let row = client.query_opt("SELECT id FROM words WHERE text = $1 AND language = $2", &[&text, &language])?;

    Ok(row.map(|r| r.get("id")))
}

pub fn get_id_words(client: &mut Client, words: &[String], language: &str) -> Result<Vec<Option<i32>>, Error> {
    let texts: Vec<&str> = words.iter().map(|s| s.as_str()).collect();

    let query = "SELECT text, id FROM words WHERE text = ANY($1) AND language = $2";
    let rows = client.query(query, &[&texts, &language])?;

    let mut result = vec![None; words.len()];
    for row in rows {
        let text: &str = row.get("text");
        let id: i32 = row.get("id");
        if let Some(pos) = words.iter().position(|w| w == text) {
            result[pos] = Some(id);
        }
    }

    Ok(result)
}

pub fn get_text_word(client: &mut Client, id: i32) -> Result<Option<String>, Error> {
    let row = client.query_opt("SELECT text FROM words WHERE id = $1", &[&id])?;
    Ok(row.map(|r| r.get("text")))
}

pub fn update_word_text(client: &mut Client, id: i32, new_text: &str) -> Result<Option<i32>, Error> {
    let row = client.query_opt("UPDATE words SET text = $1 WHERE id = $2", &[&new_text, &id])?;
    Ok(row.map(|r| r.get("id")))
}

pub fn update_word_language(client: &mut Client, id: i32, new_language: &str) -> Result<Option<i32>, Error> {
    let row = client.query_opt("UPDATE words SET language = $1 WHERE id = $2", &[&new_language, &id])?;
    Ok(row.map(|r| r.get("id")))
}

pub fn delete_word(client: &mut Client, id: i32) -> Result<u64, Error> {
    let count = client.execute("DELETE FROM words WHERE id = $1", &[&id])?;
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
    fn test_create_word() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let id = insert_word(&mut client, "unitcreate", "english").unwrap();
        assert!(id > 0);

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_insert_multiple_words() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let words = vec!["new1".to_string(), "new2".to_string(), "exist1".to_string()];

        insert_word(&mut client, "exist1", "english").unwrap();

        let inserted_ids = insert_multiple_words(&mut client, &words, "english").unwrap();

        assert_eq!(inserted_ids.len(), 2);
        assert!(inserted_ids.iter().all(|&id| id > 0));

        let id_new1 = get_id_word(&mut client, "new1", "english").unwrap();
        let id_new2 = get_id_word(&mut client, "new2", "english").unwrap();
        assert!(inserted_ids.contains(&id_new1.unwrap()));
        assert!(inserted_ids.contains(&id_new2.unwrap()));

        let id_exist1 = get_id_word(&mut client, "exist1", "english").unwrap();
        assert!(!inserted_ids.contains(&id_exist1.unwrap()));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_get_existing_words() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let words = vec!["exist1".to_string(), "exist2".to_string(), "notexist".to_string()];
        
        insert_word(&mut client, &words[0], "english").unwrap();
        insert_word(&mut client, &words[1], "english").unwrap();

        let existing = get_existing_words(&mut client, &words, "english").unwrap();

        let expected: HashSet<String> = vec!["exist1".to_string(), "exist2".to_string()].into_iter().collect();
        assert_eq!(existing, expected);

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_read_word_id() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let word = "unitread";

        let id = insert_word(&mut client, &word, "english").unwrap();
        let fetched = get_id_word(&mut client, &word, "english").unwrap();
        assert_eq!(fetched, Some(id));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_read_id_words(){
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let words = vec!["test1".to_string(), "test2".to_string(), "test3".to_string()];

        let ids = vec![
            insert_word(&mut client, &words[0], "english").unwrap(),
            insert_word(&mut client, &words[1], "english").unwrap(),
        ];

        let fetched = get_id_words(&mut client, &words, "english").unwrap();

        assert_eq!(fetched, vec![Some(ids[0]), Some(ids[1]), None]);

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_read_word_text() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let id = insert_word(&mut client, "unitread", "english").unwrap();
        let fetched = get_text_word(&mut client, id).unwrap();
        assert_eq!(fetched, Some("unitread".to_string()));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_update_word_text() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let old_text = "unitup".to_string();
        let language = "english".to_string();
        let new_text = "updated".to_string();

        let id = insert_word(&mut client, &old_text, &language).unwrap();
        update_word_text(&mut client, id, &new_text).unwrap();

        let updated_id = get_id_word(&mut client, &new_text, &language).unwrap();
        assert_eq!(updated_id, Some(id));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_update_word_language() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let text = "unitup".to_string();
        let old_language = "english".to_string();
        let new_language = "português".to_string();

        let id = insert_word(&mut client, &text, &old_language).unwrap();
        update_word_language(&mut client, id, &new_language).unwrap();

        let updated_id = get_id_word(&mut client, &text, &new_language).unwrap();
        assert_eq!(updated_id, Some(id));

        client.batch_execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_delete_word() {
        let mut client = connect_test_client();
        client.batch_execute("BEGIN").unwrap();

        let text = "unitup".to_string();
        let language = "english".to_string();

        let id = insert_word(&mut client, &text, &language).unwrap();
        let count = delete_word(&mut client, id).unwrap();
        assert_eq!(count, 1);

        client.batch_execute("ROLLBACK").unwrap();
    }
}

