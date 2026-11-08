# Company factory on the VPS

This page is the AIFluxx company factory. It is not Warp Factories and it is not the headless terminal in `deploy/vps`. Do not publish port 8791. Do not point `factory.aifluxx.co.za` at the terminal.

Nothing in this directory changes DNS, creates a database, or edits another service. The parent runs the commands after the local signed-in browser check of this revision.

## What the parent fills in

Copy `deploy/factory/.env.example` to `deploy/factory/.env` on the VPS. Mode `600`. Do not commit it.

| Name | Value |
| --- | --- |
| `FACTORY_BIND` | `0.0.0.0:8792` inside the container. The compose port stays `127.0.0.1:8792`. |
| `FACTORY_PUBLIC_URL` | `https://factory.aifluxx.co.za` |
| `FACTORY_SESSION_SECRET` | 32 random bytes, `openssl rand -base64 32`. New secret, not a Warp key. |
| `FACTORY_COOKIE_SECURE` | `1` |
| `FACTORY_REPO_ID` | `warp` |
| `FACTORY_REPO_ROOT` | `/srv/factory-repo` |
| `OIDC_ISSUER` | `https://accounts.google.com` |
| `OIDC_CLIENT_ID` | From the new OAuth client below |
| `OIDC_CLIENT_SECRET` | From that same client. One line. No quotes. |
| `FACTORY_ALLOWED_DOMAIN` | `aifluxx.co.za`, or the Workspace primary domain if it differs |

Google sign-in requirements already implemented: scope `openid email`, RS256 only, redirect `{FACTORY_PUBLIC_URL}/callback`, and a token check that the hosted domain (`hd`) and the email domain both equal `FACTORY_ALLOWED_DOMAIN` and that `email_verified` is true. A Gmail account has no company `hd` and is refused. The `hd` value sent to Google is only an account-chooser hint. An address on a different domain, including a subdomain, is refused even when `hd` matches. Set `FACTORY_ALLOWED_DOMAIN` to the Workspace primary domain that appears on the company email.

## Google client secret

In the company's existing Google Cloud project, not a new organization and not an existing SSO client:

1. APIs & Services, OAuth consent screen. Leave every other app as it is. If the consent screen can be Internal, use Internal for this client only.
2. Credentials, Create credentials, OAuth client ID, Web application.
3. Name: `AIFluxx company factory`.
4. Authorized redirect URIs, both of them:
   - `http://127.0.0.1:8792/callback`
   - `https://factory.aifluxx.co.za/callback`
5. Copy the client id and client secret into the untracked `.env`. Store the secret only there.

## Build and start

Build away from the VPS if memory is tight. The image does not include the terminal.

```bash
cd deploy/factory
cp .env.example .env
# fill .env, then:
chmod 600 .env
docker compose build
docker save company-factory:1 | gzip > company-factory-1.tar.gz
```

On `srv1307450` (`72.61.146.253`), confirm `company-factory` and port 8792 are free, and that 80 and 443 still belong to the existing proxy:

```bash
docker ps -a --format '{{.Names}}' | grep -E 'company-factory|warp-oss|aifluxx' || true
ss -ltn | grep -E ':8792\b|:8791\b|:443\b' || true
gunzip -c company-factory-1.tar.gz | docker load
cd deploy/factory
docker compose up -d --no-build
```

Put one visible file in the repo volume if it is empty:

```bash
docker run --rm -v company-factory-repo:/srv/factory-repo alpine \
  sh -c 'echo visible > /srv/factory-repo/notes.txt'
```

The container reads that volume read-only.

## HTTPS

Add one vhost on the proxy that already owns 443. Forward only `factory.aifluxx.co.za` to `http://127.0.0.1:8792`. Do not add a second listener on 443. Do not forward this name to port 8791.

DNS, after the vhost answers on loopback: an A record `factory` to `72.61.146.253`. This repository does not create that record.

## Health check

```bash
curl -fsS http://127.0.0.1:8792/healthz
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8792/workspace
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8791/ || true
```

`/healthz` is `{"status":"ok","service":"aifluxx-company-factory"}`. `/workspace` is `401`. Port 8791 must not be the page you just proxied.

Then from a browser, open `https://factory.aifluxx.co.za`, sign in with an `@aifluxx.co.za` Workspace account, and confirm a personal Gmail account is refused.

## Rollback

```bash
cd deploy/factory
docker compose stop
docker compose rm -f
```

Remove only the `factory.aifluxx.co.za` vhost. Delete the A record if it was added. Do not stop `aifluxx-dev`, `aifluxx-enterprise-dev`, or `warp-oss-tui`. `docker compose down` in this directory removes only `company-factory-net`. It does not take a volume flag unless you also mean to delete `company-factory-repo`.
