## oz

Encrypted team secrets manager backed by Cloudflare Workers and D1. The repo includes:

- **API + web UI** in `crates/oz-api` (Rust Worker with an embedded React UI)
- **CLI** in `crates/oz-cli` for scripts and local workflows
- **Shared types** in `crates/oz-core`

Sign in to the web UI with GitHub to create projects, manage members, and issue API keys. Use those keys with the CLI or `/v1` HTTP API to read and write secrets from CI, scripts, or your laptop.

## Usage

### Web UI

1. Open the deployed Worker URL (or `http://localhost:8787` when running locally).
2. Click **Sign in with GitHub**.
3. Create a project (slug + display name).
4. Create an API key scoped to a project with `read` or `write` permission. The full key is shown once — copy it before leaving the page.
5. Add, reveal, and delete secrets from the UI.

Session-authenticated requests that change data require a CSRF token (`GET /api/csrf`, then send `X-CSRF-Token` on `POST`/`PUT`/`DELETE`). API key requests skip CSRF.

### CLI

Install from source:

```bash
cargo install --path crates/oz-cli --force
```

Or download a prebuilt binary from [GitHub Releases](https://github.com/geoffsee/oz/releases) (`cli-<version>-<target>.tar.gz`).

Configure credentials (saved to `~/.config/oz/config.toml`):

```bash
oz auth login --api-key oz_live_... --api-url https://your-oz-api.example.com
```

`OZ_API_URL` and `OZ_API_KEY` override the saved config for a single invocation. The default API URL is `http://localhost:8787`.

```bash
# List projects visible to the API key
oz project list

# List secret names (not values)
oz secrets list --project my-app

# Read a secret value (printed to stdout)
oz secrets get DATABASE_URL --project my-app

# Write a secret
oz secrets set DATABASE_URL --project my-app "postgres://..."

# Write from stdin (useful for multiline or sensitive input)
echo -n "super-secret" | oz secrets set API_TOKEN --project my-app --from-stdin

# Delete a secret
oz secrets delete OLD_KEY --project my-app

# Remove saved credentials
oz auth logout
```

API keys must start with `oz_live_` and be at least 32 characters after the prefix.

### HTTP API (`/v1`)

Use a project-scoped API key as a Bearer token:

```bash
curl -H "Authorization: Bearer oz_live_..." \
  https://your-oz-api.example.com/v1/projects

curl -H "Authorization: Bearer oz_live_..." \
  https://your-oz-api.example.com/v1/projects/my-app/secrets/DATABASE_URL

curl -X PUT -H "Authorization: Bearer oz_live_..." \
  -H "Content-Type: application/json" \
  -d '{"value":"postgres://..."}' \
  https://your-oz-api.example.com/v1/projects/my-app/secrets/DATABASE_URL
```

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/projects` | List projects the key can access |
| `GET` | `/v1/projects/{slug}/secrets` | List secret metadata (key names and versions) |
| `GET` | `/v1/projects/{slug}/secrets/{key}` | Read a secret value |
| `PUT` | `/v1/projects/{slug}/secrets/{key}` | Create or update a secret (`{"value":"..."}`) |
| `DELETE` | `/v1/projects/{slug}/secrets/{key}` | Delete a secret |

The same routes exist under `/api/projects/...` for the browser UI (session auth). Prefer `/v1` for automation.

## Local development

**Prerequisites:** Rust, [Bun](https://bun.sh), and [Wrangler](https://developers.cloudflare.com/workers/wrangler/).

1. Copy or create `crates/oz-api/wrangler.toml` (see below) and `crates/oz-api/.dev.vars` (see secrets section).
2. Apply D1 migrations to the local database:

```bash
cd crates/oz-api
wrangler d1 migrations apply oz-test --local -e test
```

3. Start the Worker in test mode (stub GitHub OAuth — no real GitHub app needed):

```bash
wrangler dev -e test
```

4. Open `http://localhost:8787`, sign in (test mode accepts any OAuth callback), create a project and API key, then use the CLI against localhost:

```bash
oz auth login --api-key oz_live_... --api-url http://localhost:8787
oz secrets set MY_KEY --project my-app "hello"
```

The web UI is rebuilt automatically when you compile `oz-api` (`build.rs` runs `bun run build` in `apps/web`). For UI-only iteration:

```bash
cd apps/web
bun run dev
```

For production-like local OAuth, run `wrangler dev` without `-e test` and use real GitHub OAuth credentials in `.dev.vars`.

Deploy:

```bash
cd crates/oz-api
wrangler d1 migrations apply oz --remote
wrangler deploy
```

## `wrangler.toml` requirements

The Worker is configured by `crates/oz-api/wrangler.toml`. These values must be correct for your environment:

- `name`: Worker name.
- `main`: built Worker entrypoint (`build/index.js`).
- `compatibility_date`: Cloudflare compatibility date.
- `[build].command`: Rust Worker build command (`worker-build`).

#### D1 binding

Under `[[d1_databases]]`:

- `binding` should stay `DB` (code expects this binding name).
- `database_name` should match your D1 database.
- `database_id` must be your real D1 database UUID.
- `migrations_dir` should point to `../../migrations`.

#### Runtime vars

Under `[vars]`:

- `OZ_BASE_URL`: public base URL for this API deployment.

#### Test environment

`[env.test]` and related sections define local test values (`OZ_ENV=test`, test GitHub API base, fake test D1 id, and test secrets). Test-mode OAuth stubs are only active when `OZ_ENV=test`.

## Secrets and local dev values

Do **not** put secrets in `wrangler.toml` for normal environments. Use secrets / dev vars instead.

For local development, `crates/oz-api/.dev.vars` should contain:

- `GITHUB_CLIENT_ID`
- `GITHUB_CLIENT_SECRET`
- `OZ_MASTER_KEY` (base64 key material)
- `OZ_API_KEY_PEPPER`
- `D1_DATABASE_ID`

## Notes

- `wrangler.toml` is gitignored, so each developer keeps local/project-specific values.
- Keep `binding = "DB"` unchanged unless you also update the Worker code.
