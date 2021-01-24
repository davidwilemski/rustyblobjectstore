## What is this?

rustyblobjectstore is a very minimal HTTP blob storage service.

Currently, all blobs are stored locally within a Sqlite database for simplicity. Other backends to external storage services or the local filesystem could be added in the future.

## API

- `POST /`: POSTing some data to the root of the server will create a new blob. The key will be the SHA-256 hash of the content and will be returned in the body of the response. Blob hashes that already exist will not be overwritten: the server will still respond with a 200 status and the hex digest of the data POSTed but will not re-record the data.

- `GET /:hexdigest`: A GET request with the path being the blob's SHA-256 hash in hexadecimal. The server will respond with either a 200 status and the body content being the blob data or with 404 in the case the blob was not found. 

There is currently no authentication or authorization. This service is meant to be an internal, non-public API that is used by upstream service(s) that do impose correct authentication and/or authorization.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
