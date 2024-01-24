# samaritan-db

Empowering data self-sovereignty

## Definition

Samaritandb is a decentralized DBMS that aims to give digital users control and soverignty over their data stored across applications on the internet.

## Goals for this prototype

- Build a simple document database that allows: ✅
  - creation/removal of databases
  - creation/update/removal of documents
- Data local is owned by a DID at all times ❌
- Sensitivity to onchain data access changes ❌
- User data collation and viewing ❌
- No censorship ❌
- Distributed networking: ❌
  - sharding
  - replication
  - use of decentralized protocols
- High CAP factor ❌
- Permissioned data Sharing between applications ❌

## Goal 1

For the achievement of goal one, two crates were simply used:

- Rocket: Rocket is used to handle the http networking aspect of the database. The database was inspired by couchDB and is build to be almost completely RESTful. Rocket helps us substancially in achieving this goal.
- Sled: Sled is used as the underlying data-store which handles all data storage and retrieval operations. It is a high performant embedded database that uses a B+ Tree engine.

### Steps in running the database

1. Generate an `application DID` onchain here.
1. Clone this repo
1. In the root folder, find the `config.ini` file. This file contains many information that enables the database to be constomized and run properly. Edit it only if you know what you're doing.
1. Clear the `.data` directory in the root folder, except the `.dbs` folder. This directory is the default path for disk data storage.
1. (Optionally) edit the `Rocket.toml` file. This file contains configurations for the http networking aspect of the database e.g the tcp port address.
1. Initialize the application that controls the database and owns the data. This is done with the `_auth` route. You'll see how to do that below.
1. Compile and run
1. Start making requests

### Http routes available

Please note that since this is a RESTful database, most of the request use methods that cant be easily fired on the browser. The `curl` utility on your terminal is the most appropriate to use.

- **index**:

  - `method`: `GET`
  - `route`: `/`
  - `auth`: None
  - `function`: It says hi to the database. Generally used to ensure database is running on specified address.
  - `request (example)`:
    ```
        curl -X PUT http://127.0.0.1:1509/
    ```
  - `response (example)`:
    ```
        200 Ok {"application_did":"","samaritandb":"Hello Explorer","vendor":{"name":"Algorealm, Inc."},"version":"0.1"}
    ```

- **initialize application**:

  - `method`: `POST`
  - `route`: `/_auth`
  - `auth`: None
  - `function`: This is one of the most important routes. It gives control of the database subsequently and exclusively to the application. If configured, it immediately kicks off synchronization with peers and tries to be up to date. It is crucial for the database to respond to onchain state changes.
  - `request (example)`:
    ```
        curl -X POST http://127.0.0.1:1509/_auth_ -H "Content-Type: application/json" \
        -d '{ "did": "did:sam:apps:3e7a1f9c4b8083d2cf63b8b1897d02c9f7bc75b0316bdaf",
              "secret": "apple banana chair dog elephant forest green happy ice jelly kite"
            }'
    ```
  - `response (example)`:
    ```
        200 Ok { "ok":true, "secret": "cf63b8b1897d02" }
    ```
  - `response (error)`:

    ```
        500 InternalServerError:
            - DID parse error occurs
            - failed to write to config file

        401 Unauthorized:
            - an application has already been intialized into the database
    ```

- **create database**:

  - `method`: `PUT`
  - `route`: `/<database_name>`
  - `auth`: Basic
  - `function`: This routes creates a database on success.
  - `request (example)`:
    ```
        curl -X PUT http://<username>:<password>@127.0.0.1:1509/first_database
    ```
  - `response (example)`:
    ```
        201 Created {"ok":true}
    ```
  - `response (error)`:

    ```
        500 InternalServerError:
            - failed to create database

        409 Conflict:
            - The database already exists
    ```

- **delete database**:

  - `method`: `DELETE`
  - `route`: `/<database_name>`
  - `auth`: Basic
  - `function`: This routes creates a database on success.
  - `request (example)`:
    ```
        curl -X DELETE http://<username>:<password>@127.0.0.1:1509/first_database
    ```
  - `response (example)`:
    ```
        200 Ok {"ok":true}
    ```
  - `response (error)`:

    ```
        500 InternalServerError:
            - failed to delete database

        404 Not Found:
            - The database does not exist on machine
    ```

- **all databases**:

  - `method`: `GET`
  - `route`: `/_all_dbs`
  - `auth`: None
  - `function`: This routes returns a list of all the databases.
  - `request (example)`:
    ```
        curl -X GET http://127.0.0.1:1509/_all_dbs
    ```
  - `response (example)`:
    ```
        200 Ok ["napoleon_history","people","plankton","relay"]
    ```
  - `response (error)`:
    ```
        500 InternalServerError:
            - failed to return list
    ```

- **uuids**:

  - `method`: `GET`
  - `route`: `/_uuids?<count>`
  - `auth`: None
  - `function`: This routes returns a list of all UUIDs (adviced) to be used as document IDs.
  - `request (example)`:
    ```
        curl -X GET http://127.0.0.1:1509/_uuids?count=3
    ```
  - `response (example)`:
    ```
        200 Ok ["39727b01-77e1-4825-bbce-1bfecc824b2e","3d4a443f-a6e0-4f84-9268-90e9fc51d175","6b7558a9-4b62-49c6-ba94-a060645e7024"]
    ```

- **update database**

  - `method`: `PUT`
  - `route`: `/<database_name>/<document_id>`
  - `auth`: Basic
  - `function`: This routes helps to update a document in the database.
  - `request (example)`:

    ```
       curl -X PUT http://<username>:<password>@127.0.0.1:1509/people/0378f893-e48d-4b69-b821-7a3c2ea7b4b1 \
        -H "Content-Type: application/json" -H "X-DID: did:sam:root:3e7a1f9c4b8083d2cf63b8b1897d02c9f7bc75b0316bdaf2"
        -d '{"data": { "name":"Victoria Temilade Adekunle", "role_model":"Martin Luther King", "complexion":"fair", "_rev": "1-d3621aab8cbcec74b10202ac75ca98cb"}}'
    ```

  - `response (example)`:
    ```
        200 Ok {"id": "0378f893-e48d-4b69-b821-7a3c2ea7b4b1", "ok":true, "rev": "2-e4a21aab8cb8fa74b10202ac75ca98cb"}
    ```
  - `response (error)`:

    ```
        500 InternalServerError:
            - write operation failed

        409 Conflict:
            - the rev field failed to match
            - the DID specified in the header does not match the one in memory

        404 Not Found:
            - the database does not exist on machine

        400 BadRequest,
            - invalid or missing X-DID header
    ```

  - `header`:
    The X-DID header is used to associate a user DID with the piece of data being stored. If it is absent or incorrect, a 400 error is returned and the data cannot be saved to the database. It is crucial that every piece of data is associated with a valid DID.
  - `revisions`:
    Revisions are useful to prevent conflict in data update. With the right `_rev` field, the database is sure that you're pointing to the latest document and are up to date. This goes a long way in conflict resolution. The `_rev` field is not included in the first write request, only subsequently when the database has returned a rev ID on write. This rev ID must then be included in the next request.

- **read document**

  - `method`: `GET`
  - `route`: `/<database_name>/<document_id>`
  - `auth`: Basic
  - `function`: This routes fetches a document in the database.
  - `request (example)`:

    ```
    curl -X GET http://<username>:<password>@127.0.0.1:1509/people/0378f893-e48d-4b69-b821-7a3c2ea7b4b1
    ```

  - `response (example)`:
    ```
        200 Ok { "id":"0378f893-e48d-4b69-b821-7a3c2ea7b4b2", "complexion":"fair", "name":"Victoria Adekunle","role_model":"Martin Luther King",  "_rev":"1-5ac8ff0a3c7aa4d4c3a39c316560fa7e" }
    ```
  - `response (error)`:

    ```
        500 InternalServerError:
            - read operation failed

        404 Not Found:
            - the document does not exist
            - the database does not exist
    ```

- **delete document**

  - `method`: `DELETE`
  - `route`: `/<database_name>/<document_id>`
  - `auth`: Basic
  - `function`: This routes deletes a document in the database.
  - `request (example)`:

    ```
        curl -X DELETE http://<username>:<password>@127.0.0.1:1509/people/0378f893-e48d-4b69-b821-7a3c2ea7b4b1
    ```

  - `response (example)`:
    ```
        200 Ok { "ok": true }
    ```
  - `response (error)`:

    ```
        500 InternalServerError:
            - delete operation failed

        404 Not Found:
            - the database does not exist
    ```

#### Basic Auth
The basic authentication authenticates the username and password and permits the request to be processed if it passes. The username is gotten from the applications DID. It is the suffix SS58 address to the application DID. The password is the value of the `secret` key returned on the `/_auth` route during application initialization. 

##### Example
        curl -X DELETE http://3e7a1f9c4b8083d2cf63b8b1897d02c9f7bc75b0316bdaf:A79rXprc0L@127.0.0.1:1509/people/0378f893-e48d-4b69-b821-7a3c2ea7b4b1