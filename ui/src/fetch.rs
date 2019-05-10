
use serde::Deserialize;
use serde_json::from_str;
use draco::fetch::*;

#[derive(Debug)]
pub enum Fetch<T> {
    NotFetched,
    Fetching,
    Fetched(T),
    Failed(String)
}

impl<'a, T: Deserialize<'a>> Default for Fetch<T> {
    fn default() -> Self {
        Fetch::NotFetched
    }
}

pub enum Message<T> {
	Start(String),
	Finish(Result<T, Error>)
}

trait FetchMessage<T> {
	fn messsage(&self) -> &Message<T>;

}

impl<'a, T: Deserialize<'a>> Fetch<T> {
	pub fn update<M: FetchMessage<T>>(&mut self, mailbox: &draco::Mailbox<M>, m: Message<T>, f: &Fn(FetchMessage<T>) -> Box<M>) {
		match m {
			Message::Start(url) => {
				mailbox.spawn(get(&url).send::<Text>(), |r| f(Message::Finish(r)))
				
			}
			FetchMessage::Finish(Ok(result)) => {
				Fetch::Fetched(from_str(&result))
			}
			FetchMessage::Finish(Err(error)) => {
				Fetch::Failed(error)
			}
		}
	}
}
