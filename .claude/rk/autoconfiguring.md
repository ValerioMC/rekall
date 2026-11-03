## Autoconfiguring

`/rk project:lightcraft autoconfigure` means: learn from the project's repository, and what it
publishes about itself, how it is built and how it takes a contribution, and store that as the
project's sections, so that every later session on the project follows it.

1. Drop the `autoconfigure` term and call `rekall_context` with the anchors that are left. You change
   no code, no checklist and no task.
2. Find the upstream repository: a GitHub or GitLab binding in `<connections>` (its target is the
   repository), else the `origin` remote of the repository in the working directory, else a
   repository URL in the project's description. If there is none, say so and stop.
3. Read what the project publishes, never your memory of it: the README, `CONTRIBUTING`,
   `CODE_OF_CONDUCT`, `SECURITY`, the issue and pull request templates (`.github/ISSUE_TEMPLATE/`,
   `.github/PULL_REQUEST_TEMPLATE*`, `.gitlab/issue_templates/`, `.gitlab/merge_request_templates/`),
   the CI workflows, the build and test files, the changelog or release notes. Read the local clone
   first; what it lacks, through the connection's wrapper (`rekall-github --connection <label> GET
   /repos/:project/contents/CONTRIBUTING.md`, `GET /repos/:project/labels`, `GET
   /repos/:project/pulls?state=closed`) or the project's public pages with WebFetch. A few recently
   merged pull requests show what no file states: how branches are named, how titles read, which
   labels are set, whether the issue is linked.
4. Read the sections the project already has with `rekall_read_sections`. One on the same subject is
   rewritten with `replace: true`, keeping everything it says that is still true: those may be my
   words, and nothing keeps the text a replace overwrites.
5. Write each section with `rekall_project_section`, one call each:
   - `Overview`: what the project is, its parts and stack, where things live, who maintains it.
   - `Contributing`: the procedure from finding work to a merged change, numbered. How an issue is
     filed (the template, its required fields, the labels), how a branch is named and from which base,
     whether to fork or push directly, the commit message rules, a sign-off or CLA, how the pull
     request is opened (title, the template's fields filled in, labels, reviewers, draft or not, the
     issue linked with `Fixes #n`) and which checks must pass. Where a connection reaches the
     repository, give the exact call for the issue and for the pull request
     (`rekall-github --connection <label> POST /repos/:project/issues -d '{"title": …, "labels": […]}'`),
     with every field the project lets you decide in advance already filled.
   - `Development`: how to set up, build, test and lint, as the project runs it, and the practices a
     change follows (style, tests, docs).
   - `Release`: only when the project publishes how it releases.

   Each rule says where it was read. A rule nothing states is left out, and named in step 7.
6. If no GitHub or GitLab connection with a credential reaches the repository, `Contributing` still
   says how to contribute, and you tell me the connection to create and bind so a session can file
   issues and open pull requests itself: its kind, its server URL and the repository as the target.
7. Say in a few lines which sections you wrote or rewrote, and what the project leaves undecided.
   Then stop: no wrapup, no step, no commit.
