# TODO

## 待办（回家后处理）

### 配置本 repo 的 git 署名（不污染全局）

当前全局 git 配置用的是本名 + 工作邮箱：
- `user.name = `
- `user.email = `

Aura 这个 repo 不应该用真实身份，改成 GitHub noreply 邮箱：

```bash
cd /path/to/Aura
git config user.name "Lisolia"
git config user.email "71111266+funnymdgoupee@users.noreply.github.com"
```

跑一次后，本 repo 所有新 commit 自动用这个署名，全局不动。

### 历史回溯（可选）

当前 4 条 commit（`b89a4f8` `40ffd08` `540bc79` `ee3f6f4`）已 push，作者还是"本名"。要擦干净需要 `git filter-repo` 重写 + force push：

```bash
pip install git-filter-repo
cd /path/to/Aura
git filter-repo --commit-callback '
    commit.author_name = b"funnymdgoupee"
    commit.author_email = b"71111266+funnymdgoupee@users.noreply.github.com"
    commit.committer_name = b"funnymdgoupee"
    commit.committer_email = b"71111266+funnymdgoupee@users.noreply.github.com"
'
git push --force origin master
```

⚠️ force push 会覆盖远端历史，确认没有别人 clone 这个 repo 再做。

## 开发任务候选

按优先级：

- [ ] iPhone 端 SwiftData 缓存 — App 杀掉后历史不丢
- [ ] watchOS Target — Apple Watch 接入，收 summary
- [ ] 多模态 — Mac 端图片/文件拖拽上传，发给 AI
- [ ] 跨会话全文搜索 — Markdown + ripgrep
- [ ] 多会话管理 UI — 重命名 / 删除 / 固定
- [ ] iPhone 语音输入 — AVSpeechRecognizer
- [ ] Phase 4 中继服务器（任意语言，独立 repo）
