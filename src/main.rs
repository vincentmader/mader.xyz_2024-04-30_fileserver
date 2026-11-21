#[macro_use]
extern crate rocket;
use rocket::fs::{relative, NamedFile};
use std::path::{Path, PathBuf};

#[get("/<path..>", rank = 10)]
pub async fn file(path: PathBuf) -> Option<NamedFile> {
    let path = Path::new(relative!("static")).join(path);
    // A folder opens like a file on Linux and only fails once the response is being written,
    // which reaches the client as a broken 200. Folders are not served, so answer 404.
    if !rocket::tokio::fs::metadata(&path).await.ok()?.is_file() {
        return None;
    }
    NamedFile::open(path).await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![file])
}

#[cfg(test)]
mod tests {
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    fn client() -> Client {
        Client::tracked(super::rocket()).expect("valid rocket instance")
    }

    #[test]
    fn serves_a_file() {
        let client = client();
        let response = client.get("/README.md").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(response.into_bytes().is_some_and(|body| !body.is_empty()));
    }

    #[test]
    fn folders_are_not_found() {
        let client = client();
        for path in ["/", "/pdf", "/pdf/", "/pdf/theses/"] {
            let response = client.get(path).dispatch();
            assert_eq!(response.status(), Status::NotFound, "{path}");
        }
    }

    #[test]
    fn missing_files_are_not_found() {
        let client = client();
        let response = client.get("/does-not-exist.pdf").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }
}
