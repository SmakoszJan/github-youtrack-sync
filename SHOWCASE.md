The following is a repository called `test` with the following issues.

<img width="1519" height="455" alt="image" src="https://github.com/user-attachments/assets/b07d7671-5473-4a75-b8fa-97223e0562f1" />

After creating a new YouTrack instance on `https://testsync.youtrack.cloud` with a project `test`, and running the command

```
cargo run -- SmakoszJan test https://testsync.youtrack.cloud test
```

the issues from GitHub show up in the specified project:

<img width="1631" height="445" alt="image" src="https://github.com/user-attachments/assets/3e9876a1-34f6-4b85-b124-38c5b1c82dbb" />

Issue B with a specified description:

<img width="943" height="298" alt="image" src="https://github.com/user-attachments/assets/182fe0fa-f7b0-40b6-aa28-9db740fffb44" />

is turned into a similar structure on YouTrack:

<img width="737" height="251" alt="image" src="https://github.com/user-attachments/assets/a52049c6-34f6-4a65-a3a4-ebecb3518feb" />

After changing the title of issue C and closing A:

<img width="525" height="306" alt="image" src="https://github.com/user-attachments/assets/fb493754-3f07-4b03-9455-a14a5e7ea43d" />

The YouTrack project gets correctly updated:

<img width="1410" height="129" alt="image" src="https://github.com/user-attachments/assets/2dd1ee7b-821e-46ae-80e7-02b25746b788" />
