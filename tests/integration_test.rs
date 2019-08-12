extern crate grammarbot;
use grammarbot::{Client, Match, Result};

#[test]
fn test_check() -> Result<()> {
    let client = Client::new(env!("API_KEY"));
    let response = client.check("I can't remember how to go their.")?;
    match response.matches[0] {
        Match {
            offset: 27,
            length: 5,
            ..
        } => Ok(()),
        _ => panic!("wrong response"),
    }
}
