# Company factory on the VPS

This page is a company sign-in and a workspace file-name list. It is not Warp Factories and it does not run a Warp agent lifecycle. It is not the headless terminal in `deploy/vps`.

The public name is `https://factory.aifluxx.co.za`. The Google hosted domain is `aifluxx.com`. Those are different names. An `@aifluxx.co.za` address is refused.

Nothing in this directory changes DNS, creates a database, or edits another service. Do not merge or start this on the VPS until a parent-computer browser has signed in with this revision.

`/login` returns 503 until the four sign-in settings below are present. That response is the unconfigured page. It does not create an account.

## Settings

Copy `deploy/factory/.env.example` to `deploy/factory/.env` on the host. Mode `600`. Do not commit it. Do not put the secret in this file's committed copy, in the image, or in a chat.

| Name | Value |
| --- | --- |
| `FACTORY_BIND` | `127.0.0.1:8792` |
| `FACTORY_PUBLIC_URL` | `https://factory.aifluxx.co.za` on the VPS. For the parent browser check, `http://127.0.0.1:8792`. No path and no trailing slash. |
| `FACTORY_SESSION_SECRET` | 32 random bytes from `openssl rand -base64 32`. A new value. |
| `FACTORY_COOKIE_SECURE` | `1` on the VPS. `0` for the `http://127.0.0.1` check. An `https` public URL forces the secure cookie either way. |
| `FACTORY_REPO_ID` | `warp` |
| `FACTORY_REPO_ROOT` | `/srv/factory-repo` |
| `OIDC_ISSUER` | `https://accounts.google.com` |
| `OIDC_CLIENT_ID` | Client id of the new web client below. Not an existing production client. |
| `OIDC_CLIENT_SECRET` | Secret of that same new client. One line. No quotes. |
| `FACTORY_ALLOWED_DOMAIN` | `aifluxx.com` |

The app requests scope `openid email`, accepts RS256 only, and sets the token audience to `OIDC_CLIENT_ID`. The callback it registers with Google is `{FACTORY_PUBLIC_URL}/callback`. The token must have `email_verified` true, hosted domain `aifluxx.com`, and an email domain of `aifluxx.com`. The `hd` value sent to Google is only an account-chooser hint.

The process exits if `FACTORY_BIND` is not a loopback address. Google sign-in exits if `OIDC_CLIENT_SECRET` is empty.

## New Google client

In the company's existing Google Cloud project:

1. Do not open, edit, or copy any existing OAuth client. Do not change its redirect URIs, origins, or audience.
2. Create credentials, OAuth client ID, Web application. Name it `AIFluxx company factory`.
3. If the consent screen can be Internal for this client only, use Internal. Leave every other app as it is.
4. Authorized redirect URIs, both, and no others:
   - `http://127.0.0.1:8792/callback`
   - `https://factory.aifluxx.co.za/callback`
5. Do not add a JavaScript origin unless the console refuses to save without one. If it does, add only `https://factory.aifluxx.co.za`.
6. Put the new client id and secret in the untracked `.env` only.

## Preflight on srv1307450

Port 8792 must be free. Ports 80 and 443 stay with the existing proxy. Port 8791 stays the private terminal.

```bash
docker ps -a --format '{{.Names}}' | grep -E 'company-factory|warp-oss|aifluxx' || true
ss -ltn | grep -E ':8792\b|:8791\b|:443\b|:80\b' || true
```

Build off the VPS. The image does not include the terminal.

```bash
cd deploy/factory
cp .env.example .env
chmod 600 .env
# fill the four secrets and ids in .env, then:
docker compose build
docker save company-factory:1 | gzip > company-factory-1.tar.gz
```

On the VPS, after the parent browser check:

```bash
gunzip -c company-factory-1.tar.gz | docker load
cd deploy/factory
docker compose up -d --no-build
docker run --rm -v company-factory-repo:/srv/factory-repo alpine \
  sh -c 'echo visible > /srv/factory-repo/notes.txt'
ss -ltn 'sport = :8792'
```

`ss` must show `127.0.0.1:8792` and must not show `0.0.0.0:8792` or `[::]:8792`.

The existing HTTPS proxy forwards only `factory.aifluxx.co.za` to `http://127.0.0.1:8792`. Do not add a second listener on 443. Do not forward that name to port 8791. The A record, when added, is `factory` to `72.61.146.253`.

## Health check

```bash
curl -fsS http://127.0.0.1:8792/healthz
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8792/workspace
curl -s -o /dev/null -w '%{http_code}\n' -H 'accept: application/json' http://127.0.0.1:8792/login
```

`/healthz` is `{"status":"ok","service":"aifluxx-company-factory"}`. `/workspace` is `401`. Before the four sign-in settings exist, `/login` is `503` and its JSON `missing` array is `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `FACTORY_ALLOWED_DOMAIN`. After they exist, `/login` is a redirect to `https://accounts.google.com`.

The parent browser check, on this revision, is still required: an `@aifluxx.com` Workspace account reaches the workspace entry, and a personal Gmail account does not.

## Rollback

```bash
cd deploy/factory
docker compose stop
docker compose rm -f
```

Remove only the `factory.aifluxx.co.za` vhost. Delete the A record if it was added. Do not stop `aifluxx-dev`, `aifluxx-enterprise-dev`, or `warp-oss-tui`. `docker compose down` in this directory does not take a volume flag unless you also mean to delete `company-factory-repo`.
