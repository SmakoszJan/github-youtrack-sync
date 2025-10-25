THIS IS A SOLUTION FOR AN APPLICATION TASK FOR JETBRAINS INTERNSHIP. THIS IS NOT INTENDED FOR DAILY USE.

---

This project downloads issues from a GitHub repository and imports them to a YouTrack project.
Then, it keeps the imported issues in sync with their GitHub counterparts (only GitHub to YouTrack).

## Usage

Requires Rust to run. Compiled on Rust 1.90, but an older version should still work.


```
  cargo run -- <OWNER> <REPO> <HOST> <PROJECT>
```

The `<OWNER>` and `<REPO>` are the owner and name of the repository you want to import from. `<HOST>` is the URL
of the YouTrack instance. `<PROJECT>` is the name of the YouTrack project to import issues to. Not the full name
is necessary - a search query should be enough, the first result will be picked.

The tool requires a GitHub access token and a YouTrack access token. They can be provided as respectively
`YOUSYNC_GITHUB_TOKEN` and `YOUSYNC_YOUTRACK_TOKEN` environment variables. If they're not provided that way,
you will be prompted for them.
