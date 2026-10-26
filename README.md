# Newsletter Demo
**Warning: This s just a demo for now**
## Container Planning
![docs/archtitecture.png](docs/architecture.png)
### used Software
| Function | Software |
|-------|-----|
| Webserver| NGINX? |
| DB | PostgreSQL |
| DB library for Rust | [SQLx](https://crates.io/crates/sqlx) |
| Mail library | [lettre](https://crates.io/crates/lettre) |

## Project structure
- docker/  # All Container related files
  - subscriber/  # Conatains *subscriber* Container
    - Dockerfile # Build the Container for *subscriber*
  - mailbot/     # Conatains *mailbot* Container
    - Dockerfile # Build the Container for *mailbot*
- src
  - src/
    - subscriber.rs  # Takes Input and puts it in a DB  
    - mailbot.rs     # Automatically sends E-Mails to subscribed Inboxes
  - lib.rs       # Shared functionality between subscriber and mailbot(Input Validatio and Config)
- init-db  # Database config
  - 1-init.sql # Creates the needed table
  - 2-users.sh # Creates Postgres users with minimal permissions
  - 3-update-users.sh # Updates User Passwords, in case they changes
- templates   # HTML templates for server responses
- subscribe.html # Contans a form to register an e-mail
- Cargo.toml # Rust project config
- Cargo.lock # Dependecy lockfile
- compose.yaml # configures all needed Containers
- .secrets/  # not commited. Contains secrets for the Containers(e.g. DB Password)
- .gitignore

## Deploy
### Prerequisites
1. Config:
  - secrets 
    - `.secrets/pg-subscriber` # Postgres-password for *subscriber*
    - `.secrets/pg-mailbot` # Postgres-password for *mailbot*
    - `.secrets/postgres-master-password` # Postgres admin password
    - `.secrets/email-config`  # Contains secrets for the source e-mail
    for DB secrets:
      type `openssl rand -hex 32 > *filename*`
    for email-config:
      first line -> account email or username
    second line -> password
  - .env
    - Configure BASE_URL
    - for check-if-email-exists crate uncomment this line: `CHECK_EMAIL_EXISTS=true`. **This is known to reject valid emails**

2. Ensure correct SELinux label: `container_file_t` for all secrets and init-db contents (Only when your systems uses SELinux)
3. run `docker compose up`

## Development
Everytime your change the code and/or html templates run `docker compose build`

## Project TODO:
- [x] Input verification
  - [x] Check if email exists
  - [x] Validation E-mail
- [x] Database setup
- [x] Allow Password rotation
- [x] mailbot
- [ ] Webserver setup
- [ ] Nur einzelne Tags abonnieren
- [x] Better Error-handling
- [ ] Extenting tests (for lib crate)
- [ ] Extending Project Docs/Comments
