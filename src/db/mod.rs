pub mod users;
pub mod words;
pub mod vocab_user;

use postgres::{Client, NoTls, Error}; // W: unused import: `NoTls`

fn init_tables(client: &mut Client) -> Result<(), Error> { // W: function `init_tables` is never used
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )
    ")?;

    client.batch_execute("
        CREATE TABLE IF NOT EXISTS words(
            id SERIAL PRIMARY KEY,
            text TEXT NOT NULL,
            language TEXT NOT NULL,
            UNIQUE(text, language)
        )"
    )?;

    client.batch_execute("
        CREATE TABLE IF NOT EXISTS vocabulary_user (
            id_user INT REFERENCES users(id),
            id_word INT REFERENCES words(id),
            PRIMARY KEY (id_user, id_word)
        )"
    )?;

    Ok(())
}

