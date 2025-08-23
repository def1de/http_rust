use sqlite;
use std::sync::{Arc, Mutex};
use crate::template::MessageView;

pub struct Database {
    connection: Arc<Mutex<sqlite::Connection>>,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            connection: Arc::clone(&self.connection),
        }
    }
}

impl Database {
    pub fn new() -> Self {
        let conn = match sqlite::open("database.db") {
            Ok(conn) => conn,
            Err(e) => panic!("Error opening database: {}", e),
        };
        Database { 
            connection: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn create(&self) -> Result<(), sqlite::Error> {
        println!("Creating database schema...");
        self.connection.lock().unwrap().execute(
            "
            CREATE TABLE IF NOT EXISTS Messages (
                messageID INTEGER PRIMARY KEY,
                message_text TEXT NOT NULL,
                username TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS Users (
                userID INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                password_hash TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS Sessions (
                sessionID INTEGER PRIMARY KEY,
                userID INTEGER NOT NULL,
                session_token TEXT NOT NULL,
                expires_at DATETIME NOT NULL,
                FOREIGN KEY(userID) REFERENCES Users(userID)
            );
            ",
        )
    }

    pub fn insert_message(&self, message_text: &str, username: &str) -> Result<(), sqlite::Error> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT INTO Messages (message_text, username) VALUES (?, ?);"
        )?;
        stmt.bind((1, message_text))?;
        stmt.bind((2, username))?;
        stmt.next()?;
        Ok(())
    }

    pub fn get_user(&self, username: &str) -> Result<Option<(i64, String)>, sqlite::Error> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT userID, username FROM Users WHERE username = ?;"
        )?;
        stmt.bind((1, username))?;
        if let sqlite::State::Row = stmt.next()? {
            let user_id: i64 = stmt.read(0)?;
            let username: String = stmt.read(1)?;
            Ok(Some((user_id, username)))
        } else {
            Ok(None)
        }
    }

    pub fn check_password(&self, username: &str, password_hash: &str) -> bool {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT password_hash FROM Users WHERE username = ?;"
        ).unwrap();

        // Return false if binding fails
        if stmt.bind((1, username)).is_err() {
            return false;
        }

        if let sqlite::State::Row = stmt.next().unwrap_or(sqlite::State::Done) {
            let stored_hash: String = stmt.read(0).unwrap_or_default();
            stored_hash == password_hash
        } else {
            false
        }
    }

    pub fn add_user(&self, username: &str, password_hash: &str) -> Result<(), sqlite::Error> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT INTO Users (username, password_hash) VALUES (?, ?);"
        )?;
        stmt.bind((1, username))?;
        stmt.bind((2, password_hash))?;
        stmt.next()?;
        Ok(())
    }

    pub fn create_session(&self, user_id: i64, session_token: &str) -> Result<(), sqlite::Error> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT INTO Sessions (userID, session_token, expires_at) VALUES (?, ?, datetime('now', '+7 days'));"
        )?;
        stmt.bind((1, user_id))?;
        stmt.bind((2, session_token))?;
        stmt.next()?;
        Ok(())
    }

    pub fn validate_session(&self, session_token: &str) -> Result<Option<(i64, String)>, sqlite::Error> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT s.userID, u.username
                        FROM Sessions AS s
                        JOIN Users AS u ON u.userID = s.userID
                        WHERE session_token = ? AND expires_at > datetime('now');"
        )?;
        stmt.bind((1, session_token))?;
        if let sqlite::State::Row = stmt.next()? {
            let user_id: i64 = stmt.read(0)?;
            let username: String = stmt.read(1)?;
            Ok(Some((user_id, username)))
        } else {
            Ok(None)
        }
    }

    pub fn delete_session(&self, session_token: &str) -> Result<(), sqlite::Error> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("DELETE FROM Sessions WHERE session_token = ?;")?;
        stmt.bind((1, session_token))?;
        stmt.next()?;
        Ok(())
    }

    pub fn get_messages(&self, limit: i64) -> Result<Vec<MessageView>, sqlite::Error> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT username, message_text FROM Messages ORDER BY timestamp DESC LIMIT ?;"
        )?;
        stmt.bind((1, limit))?;
        
        let mut messages = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            let username: String = stmt.read(0)?;
            let message_text: String = stmt.read(1)?;
            messages.push(MessageView { username, text: message_text });
        }
        Ok(messages)
    }
}