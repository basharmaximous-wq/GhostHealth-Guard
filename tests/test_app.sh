# This sends a "fake" Pull Request payload to your local server
curl -X POST http://localhost:3000/webhook \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: pull_request" \
  -d '{
    "action": "opened",
    "pull_request": { "number": 15, "diff_url": "fake_url" },
    "repository": { "full_name": "test-user/health-repo" },
    "sender": { "login": "tester" }
  }'