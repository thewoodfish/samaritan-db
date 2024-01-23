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
Please not that since this is a RESTful database, most of the request use methods that cant be easily fired on the browser. The `curl` utility on your terminal is the most appropriate to use.

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
            200 Ok {"ok":true}
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

