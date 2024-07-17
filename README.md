# htmx-todo
Sample application for proving out htmx and axum as a viable web application platform

See the [Demo](https://todo.vanhouzen.me)

## Concepts
### Axum
Axum is used as the HTTP server, it uses complex types to validate handlers rather than macros like many other rust servers use saving a bit of compile time. It is a front-end wrapper around tokios hyper client which means that its very performant and able to leverage the tower ecosystem of middlewares. Many of these are used in this project including middleware for live reloading the page on project restart.

### SQLx
SQLx is a library for querying multiple different database backends. Its standout feature is the ability to compile time verify SQL queries against a provided database. This project uses SQLx with a PostgreSQL database, it also takes advantage of the migration features of SQLx to alter the database tables.

### Rinja (Formly Askama)
Rinja is an HTML templating engine for Rust that can generate dynamic HTML documents based off of a Jinja2 derivative template file. Rinja is able to check templates and their context at compile time which helps make sure that all template render correctly without manual testing. Rinja also provides convenient features for being able to render fragments of templates which synergizes very well HTMX for rendering partial DOM updates.

### HTMX
HTMX is used to turn this traditional Multiple Page Application(MPA) into an Single Page Application(SPA) with the use of special decorators applied to HTML elements. HTMX has the ability to trigger AJAX calls via any html element and swap response HTML into the DOM.

### Progressive Enhancement
This app is able to function in its entirety in the absence of javascript by only using HTMX to improve UX rather than function as critical components. This approach grants excellent accessibility and simplicity but does mean that interactable elements have to be limited to ```<a>``` tags and ```<form>``` elements. CSS styling can be used to make an ```<a>``` look like practically any element desired however ```<form>``` does have restrictions in where it can be placed, critically this makes inline forms in tables extremely difficult to implement.

### HATEOS
Hypertext As The Engine Of State(HATEOS) is a concept that espouses using the returned HTML of the application as the sole means of state transition. So rather than a JSON api each user facing data structure is represented with HTML fragments that contain all controls necessary to manipulate its state. One particular point of confusion is that HATEOS is an API structure but this is mistaken, HATEOS is a purely human concept as it relies on humans understanding the HTML representation of the applications data structures and the manipulations presented via hypermedia controls.

### Containers
This project contains a ```Dockerfile``` that will build a debian bookworm based container with the application. This makes quickly creating an instance of the app and composing it with a database very easy and repeatable.

### Fly.io
Fly.io is a service that takes container images and converts them into Firecracker VMs that it runs globally. This project is setup to push new versions to Fly.io which automatically deploys them into an environment with an attached shared PostgreSQL database.

## Building
To build the project you must have the Rust toolchain installed. The project contains SQLx compiled queries and will either offline verify them with the contents of the .sqlx folder or if ```DATABASE_URL``` is defined it will verify queries against the database provided which must be up to date and migrated.

### Dev Container
The project provides a preconfigured devcontainer that consists of an application image with the app and necessary build tools and a PostgreSQL container to use as a development database. This devcontainer can also be used from Github Codespaces.

## Contributing
Feel free to open issues or contribute pull requests for any features you feel are missing from the project!