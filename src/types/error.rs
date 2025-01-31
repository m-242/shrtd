use thiserror::Error;

/** Represent's an application's error */
#[derive(Error, Debug)]
pub enum Error {
    /* 4xx errors */
    #[error("Couldn't find the record identified with the slug `{0}`")]
    NotFound(String),

    #[error("File upload failed ({0})")]
    FileUpload(String),

    #[error("Paste creation failed ({0})")]
    PasteCreation(String),

    #[error("Redirect creation failed ({0})")]
    RedirectCreation(String),

    /* 5xx errors */
    #[error("There's an error with the configuration ({0})")]
    Config(#[from] figment::Error),

    #[error("I/O error: {0}")]
    IO(#[from] tokio::io::Error),

    #[error("Could not query the Redis server ({0})")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization or deserialization error ({0})")]
    SerDe(#[from] serde_json::Error),
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r rocket::request::Request<'_>) -> rocket::response::Result<'o> {
        use rocket::{http::Status, response::status, serde::json};
        use serde::Serialize;

        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let error = json::Json(ErrorResponse {
            message: self.to_string(),
        });

        Ok(match self {
            /* 4xx errors */
            Error::NotFound(_) => status::NotFound(error).respond_to(req)?,
            Error::FileUpload(_) | Error::PasteCreation(_) | Error::RedirectCreation(_) => {
                status::Custom(Status::UnprocessableEntity, error).respond_to(req)?
            }

            /* 5xx errors */
            Error::Redis(_) => status::Custom(Status::ServiceUnavailable, error).respond_to(req)?,
            Error::Config(_) | Error::IO(_) | Error::SerDe(_) => {
                status::Custom(Status::InternalServerError, error).respond_to(req)?
            }
        })
    }
}

/** Convenience alias of [`std::result::Result`] with the [`Error`] prefilled */
pub type Result<T, E = Error> = std::result::Result<T, E>;
