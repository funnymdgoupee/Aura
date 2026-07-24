# AI Assistant 三端 UI 设计要素文档
# 版本: v1.0 | 风格: 浅色科技 / Vercel 式极简
# 用途: 可直接将此文档交给任意 AI 进行界面复现

---

## 一、设计哲学

- **核心关键词**: Light, Airy, Precise, Minimal
- **视觉隐喻**: 清晨空气般的清透感，界面退后、内容向前
- **禁忌**: 不使用深色模式、不使用霓虹发光、不使用厚重阴影、不使用高饱和强调色
- **分隔哲学**: 全程使用 1px 细线分隔区域，不用阴影制造层级

---

## 二、色彩系统（Color Tokens）

### 2.1 背景层级

| Token | 色值 | 用途 |
|-------|------|------|
| `--bg` | `#ffffff` | 最底层背景、主内容区 |
| `--bg-elevated` | `#fafafa` | 侧边栏、面板、卡片背景 |
| `--bg-surface` | `#f5f5f7` | 输入框背景、次级表面 |

### 2.2 文字层级

| Token | 色值 | 用途 |
|-------|------|------|
| `--text-primary` | `#111111` | 标题、主文字、按钮背景 |
| `--text-secondary` | `#666666` | 正文、描述、次级信息 |
| `--text-muted` | `#a1a1aa` | 占位符、时间戳、禁用态 |

### 2.3 边框与分隔

| Token | 色值 | 用途 |
|-------|------|------|
| `--border` | `rgba(0, 0, 0, 0.06)` | 默认边框、分隔线 |
| `--border-hover` | `rgba(0, 0, 0, 0.12)` | 悬浮态边框 |

### 2.4 强调色（极度克制）

| Token | 色值 | 用途 |
|-------|------|------|
| `--accent-blue` | `#2563eb` | 代码高亮、链接、聚焦 ring |
| `--accent-purple` | `#7c3aed` | AI 思考状态、渐变辅色 |
| `--accent-pink` | `#db2777` | 极少使用，仅作渐变点缀 |
| `--accent-gradient` | `linear-gradient(135deg, #2563eb 0%, #7c3aed 100%)` | 图标、球体、头像背景 |
| `--success` | `#10b981` | 在线状态、成功提示 |

### 2.5 阴影（极少使用）

```css
/* 仅卡片悬浮时出现，极淡 */
box-shadow: 0 8px 32px rgba(0, 0, 0, 0.04);
```

---

## 三、字体系统（Typography）

### 3.1 字体栈

| 用途 | 字体 |
|------|------|
| 界面文字 | `Inter, -apple-system, BlinkMacSystemFont, sans-serif` |
| 代码/数值 | `SF Mono, JetBrains Mono, monospace` |

### 3.2 字号层级

| 名称 | 字号 | 字重 | 字间距 | 行高 | 用途 |
|------|------|------|--------|------|------|
| Display | 52px | 700 | -2px | 1.1 | 页面大标题 |
| H1 | 28px | 700 | -0.5px | 1.2 | 区块标题、欢迎语 |
| H2 | 24px | 700 | -0.5px | 1.2 | 子标题 |
| H3 | 20px | 700 | -0.5px | 1.3 | 卡片标题 |
| Body | 15px | 400 | 0 | 1.6 | 正文、消息内容 |
| Body-Small | 14px | 400 | 0 | 1.5 | 导航项、按钮文字 |
| Caption | 13px | 500 | 0 | 1.4 | 标签、时间戳 |
| Micro | 12px | 600 | 1px (uppercase) | 1.4 | 分组标题、Token 标签 |
| Mono | 13px | 400 | 0 | 1.5 | 代码块、技术参数 |

### 3.3 文字颜色规则

- 标题/主按钮: `#111111` (on white) / `#ffffff` (on black)
- 正文: `#666666`
- 占位符/辅助: `#a1a1aa`
- 代码关键字: `#2563eb` (蓝色)
- 代码背景: `rgba(0,0,0,0.05)` + 圆角 4px

---

## 四、间距系统（Spacing）

### 4.1 圆角（Radius）

| Token | 值 | 用途 |
|-------|-----|------|
| `--radius-sm` | 8px | 小按钮、输入框、导航项 |
| `--radius-md` | 12px | 卡片、历史项、用户卡片 |
| `--radius-lg` | 16px | 大卡片、输入区、二维码框 |
| `--radius-xl` | 24px | 面板、Mac 窗口、iPhone 外壳 |
| `--radius-pill` | 9999px | 标签、Tab 切换器 |

### 4.2 内边距（Padding）

| 场景 | 值 |
|------|-----|
| 页面边距 | 48px 32px (桌面) / 20px (移动端) |
| 卡片内部 | 24px - 28px |
| 导航项 | 8px 12px |
| 按钮 | 10px 20px (标准) / 8px 16px (小) / 12px (大) |
| 输入框 | 12px 16px |
| 消息气泡 | 14px 18px |
| 侧边栏 | 20px 16px |

### 4.3 外边距（Margin）

| 场景 | 值 |
|------|-----|
| 区块间距 | 80px |
| 卡片间距 | 16px |
| 元素间距 | 12px - 16px |
| 紧凑间距 | 4px - 8px |

---

## 五、布局结构（三端）

### 5.1 Mac 桌面端 — 三栏布局

```
┌─────────────────────────────────────────────────────────────┐
│  Sidebar (220px) │  Main Content (flex:1)  │  Panel (280px)│
│  #fafafa 底       │  #ffffff 底              │  #fafafa 底   │
│  右 1px 分隔线    │                         │  左 1px 分隔线│
└─────────────────────────────────────────────────────────────┘
```

**左侧边栏（Sidebar）**
- 宽度: 220px
- 背景: `#fafafa`
- 内容从上到下: Logo(28px图标+文字) → 主导航(5项) → 分隔线 → AI Tools分组(2项) → 用户卡片(底部固定)
- 导航项: 8px 12px 内边距，8px 圆角，悬浮 `#000000` 3% 透明度背景
- 当前项: `#000000` 5% 透明度背景，文字 `#111111`

**中间内容区（Main）**
- 背景: `#ffffff`
- 内边距: 32px 40px
- 顶部状态栏: 左侧会话名(13px muted) + 右侧服务器状态(6px绿点+文字)
- 欢迎页（空状态）: 居中，48px渐变星形图标 → 28px问候语 → 15px副标题 → 三列快捷卡片
- 聊天状态: 消息列表(flex:1, 16px间距) + 底部输入区

**快捷卡片（Quick Actions）**
- 三列等宽网格，12px 间距
- 卡片: `#fafafa` 底，1px 边框，16px 圆角，20px 内边距
- 图标区: 36px 圆角方块，`#000000` 4% 透明度背景，居中 16px 图标
- 标题: 14px 600字重
- 描述: 12px muted

**底部输入区**
- 通栏，相对定位
- 输入框: `#fafafa` 底，1px 边框，16px 圆角，16px 20px 内边距，右侧留 120px 给按钮
- 聚焦态: 边框升至 12% 黑 + `box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.08)` (蓝色 focus ring)
- 右侧按钮组: absolute 定位，右 12px，垂直居中
  - Attach 按钮: ghost 样式，12px 字
  - 发送按钮: 32px 圆角方块，纯黑底白字，8px 圆角

**右侧历史面板（Panel）**
- 宽度: 280px
- 背景: `#fafafa`
- 内容: 标题行(14px 600字重 + 设置图标) → 搜索框 → 历史列表 → 底部新建按钮
- 搜索框: `#ffffff` 底，1px 边框，8px 圆角，10px 14px 内边距
- 历史项: `#ffffff` 底，12px 内边距，12px 圆角，1px 透明边框，悬浮显示边框
- 历史项内容: 标题(13px 500) + 描述(11px muted) + 时间(10px muted) + 可选头像组
- 新建按钮: 通栏，纯黑底白字，12px 圆角，13px 600字重

### 5.2 iPhone 移动端

**外壳**
- 宽度: 360px（原型尺寸），实际开发用 100vw
- 背景: `#ffffff`
- 圆角: 44px（仅原型展示用）
- 阴影: `0 20px 60px rgba(0,0,0,0.08)`（仅原型）

**屏幕内容**
- 顶部: 52px 上内边距（留给 notch）→ 标题行
  - 左侧: 28px 700字重大标题 "Chat"
  - 右侧: 6px 绿点 + 13px 设备名
- 消息区: flex:1，16px 间距，垂直排列
  - 用户消息: align-self: flex-end，max-width 85%，纯黑底(`#111`)白字，18px 圆角，右下角 6px
  - AI 消息: align-self: flex-start，max-width 85%，`#fafafa` 底，1px 边框，18px 圆角，左下角 6px
  - 代码: SF Mono 12px，蓝色关键字，浅灰底 4px 圆角
- 输入区: 12px 16px 内边距，`#fafafa` 底，1px 边框，24px 圆角（胶囊形）
  - 输入框: flex:1，透明底，15px 字
  - 发送按钮: 32px 圆形，纯黑底白字

**扫码配对页**
- 全屏居中布局
- 提示文字: 15px secondary
- 二维码框: 200px 正方形，白色底，16px 圆角，1px 边框，极淡阴影
- 扫描线: 2px 高，黑色 30% 透明度，2.5s 无限循环上下扫描动画
- 底部 fallback: 13px muted "or enter IP manually"

### 5.3 Apple Watch

**外壳**
- 宽度: 220px（原型），实际开发用 100%
- 圆角: 48px
- 背景: `#ffffff`

**屏幕内容**
- 垂直居中排列，20px 间距
- AI 形象球体: 72px 直径，圆形
  - 背景: `radial-gradient(circle at 30% 30%, rgba(255,255,255,0.4), transparent 60%), linear-gradient(135deg, #2563eb, #7c3aed)`
  - 阴影: `0 4px 20px rgba(37, 99, 235, 0.15)`
  - 外圈轨道: 绝对定位，inset -8px，1px 浅灰边框(`rgba(0,0,0,0.06)`)，圆形，4s 无限旋转
- 状态文字: 13px，600字重，0.5px 字间距
- 摘要: 12px，muted 色，居中，最多两行
- 展开按钮: `#fafafa` 底，1px 边框，20px 圆角，12px 500字重

---

## 六、组件规范

### 6.1 按钮（Button）

| 类型 | 背景 | 边框 | 文字色 | 圆角 | 内边距 | 悬浮态 |
|------|------|------|--------|------|--------|--------|
| Primary | `#111111` | 无 | `#ffffff` | 8px | 10px 20px | 背景 `#333333`，translateY(-1px) |
| Secondary | 透明 | 1px `#000000` 6% | `#111111` | 8px | 10px 20px | 边框升至 12%，背景 `#000000` 2% |
| Ghost | 透明 | 无 | `#666666` | 8px | 8px 16px | 文字色 `#111111` |
| Icon Button | `#111111` | 无 | `#ffffff` | 8px | 0（固定 32x32） | scale(1.05) |

**按下态**: `transform: scale(0.98)`，无阴影变化

### 6.2 输入框（Input）

- 背景: `rgba(0,0,0,0.02)` 或 `#fafafa`
- 边框: 1px `rgba(0,0,0,0.06)`
- 圆角: 8px（桌面）/ 24px（移动端胶囊）
- 内边距: 12px 16px
- 文字: 14px-15px，`#111111`
- 占位符: `#a1a1aa`
- 聚焦: 边框升至 12% 黑 + `box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.08)`

### 6.3 卡片（Card）

- 背景: `#fafafa`
- 边框: 1px `rgba(0,0,0,0.06)`
- 圆角: 16px
- 内边距: 24px
- 悬浮: 边框升至 12%，translateY(-2px)，极淡阴影 `0 8px 32px rgba(0,0,0,0.04)`

### 6.4 消息气泡（Message Bubble）

**用户消息**
- align-self: flex-end
- max-width: 70%（桌面）/ 85%（移动端）
- 背景: `#111111`
- 文字: `#ffffff`，15px，450字重
- 圆角: 18px，右下角 6px
- 内边距: 14px 18px

**AI 消息**
- align-self: flex-start
- max-width: 70%（桌面）/ 85%（移动端）
- 背景: `#fafafa`
- 边框: 1px `rgba(0,0,0,0.06)`
- 文字: `#111111`，15px
- 圆角: 18px，左下角 6px
- 内边距: 14px 18px

### 6.5 标签（Tag）

- 背景: `rgba(0,0,0,0.02)`
- 边框: 1px `rgba(0,0,0,0.06)`
- 圆角: 9999px（胶囊）
- 内边距: 6px 16px
- 文字: 13px，`#666666`
- 悬浮: 边框升至 12%，文字 `#111111`

---

## 七、动效规范（Motion）

### 7.1 缓动曲线

| 名称 | 值 | 用途 |
|------|-----|------|
| Default | `ease-out` | 通用 |
| Entrance | `cubic-bezier(0.16, 1, 0.3, 1)` | 元素入场 |
| Exit | `cubic-bezier(0.4, 0, 0.2, 1)` | 元素退场 |
| Bounce | `cubic-bezier(0.34, 1.56, 0.64, 1)` | 弹性反馈 |

### 7.2 入场动画

**页面入场**
```css
animation: fadeIn 0.8s ease-out;
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
```
- 页面头部: 0s 延迟
- 内容区块: 0.1s 延迟递增
- 底部总结: 0.4s 延迟

**消息入场**
```css
animation: msgIn 0.35s ease-out;
@keyframes msgIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
```

**卡片悬浮**
```css
transition: all 0.3s ease;
/* hover 态 */
transform: translateY(-2px);
border-color: rgba(0,0,0,0.12);
box-shadow: 0 8px 32px rgba(0,0,0,0.04);
```

### 7.3 微交互

| 元素 | 触发 | 效果 | 时长 |
|------|------|------|------|
| 按钮 | hover | translateY(-1px) | 0.2s |
| 按钮 | active | scale(0.98) | 0.1s |
| 输入框 | focus | 边框变亮 + 蓝色 ring | 0.2s |
| 导航项 | hover | 背景 3% 黑 | 0.15s |
| 发送按钮 | hover | scale(1.05) | 0.2s |
| 历史项 | hover | 显示边框 + 极淡阴影 | 0.15s |

### 7.4 背景动效

**弥散光晕（Ambient Glow）**
- 3 个绝对定位圆球，fixed 覆盖全屏，pointer-events: none
- 圆球 1: 700px，`linear-gradient(135deg, #bfdbfe, #ddd6fe)`，top: -15%, left: -10%
- 圆球 2: 600px，`linear-gradient(135deg, #e9d5ff, #fbcfe8)`，bottom: -15%, right: -5%
- 圆球 3: 500px，`linear-gradient(135deg, #dbeafe, #fce7f3)`，top: 35%, left: 45%
- 统一: `filter: blur(140px)`，`opacity: 0.2-0.35`
- 动画: `drift`，25s infinite ease-in-out
```css
@keyframes drift {
  0%, 100% { transform: translate(0, 0) scale(1); }
  33% { transform: translate(40px, -30px) scale(1.08); }
  66% { transform: translate(-30px, 20px) scale(0.95); }
}
```
- 各圆球 animation-delay 错开: 0s, -8s, -16s

**网格纹理（Grid Texture）**
- 固定全屏，pointer-events: none
- 背景图: 60px 间距的 1px 细线网格，`rgba(0,0,0,0.025)`
- 遮罩: `radial-gradient(ellipse at center, black 0%, transparent 70%)`
- 中心清晰，边缘淡出

### 7.5 AI 形象状态机动画

| 状态 | 球体渐变 | 动画 | 阴影 | 状态文字色 |
|------|----------|------|------|------------|
| 待命 (Idle) | `#2563eb` → `#7c3aed` | breathe 3s | `0 4px 20px rgba(37,99,235,0.15)` | `#666666` |
| 倾听 (Listening) | 同上 | breathe 2s（稍快） | 同上 | `#666666` |
| 思考 (Thinking) | `#7c3aed` → `#db2777` | think 1s（快速缩放） | `0 4px 24px rgba(124,58,237,0.15)` | `#7c3aed` |
| 回应 (Responding) | `#10b981` → `#2563eb` | breathe 3s | `0 4px 20px rgba(16,185,129,0.15)` | `#10b981` |
| 执行 (Executing) | `#f59e0b` → `#ef4444` | progress 环旋转 | `0 4px 20px rgba(245,158,11,0.15)` | `#f59e0b` |

```css
@keyframes breathe {
  0%, 100% { transform: scale(1); opacity: 0.9; }
  50% { transform: scale(1.06); opacity: 1; }
}
@keyframes think {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.1); }
}
@keyframes orbRing {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
```

---

## 八、状态与图标

### 8.1 连接状态

| 状态 | 视觉 | 颜色 |
|------|------|------|
| 在线 | 6px 实心圆 | `#10b981` |
| 连接中 | 6px 圆 + 脉冲动画 | `#f59e0b` |
| 离线 | 6px 圆 | `#ef4444` |

### 8.2 图标风格

- 使用简单几何形状或 Unicode 符号（◆ ◈ ▣ ◉ ▤ ▩ ✦ ✎）
- 不使用复杂插画图标
- 尺寸: 导航 16px，标题 20px，按钮 12px
- 颜色: 默认 `#666666`，当前项/悬浮 `#111111`

---

## 九、关键实现提示

1. **不用 backdrop-filter**: 浅色模式下毛玻璃效果容易显脏，用纯色层级（#fff → #fafafa → #f5f5f7）即可表达层级
2. **不用 box-shadow 制造层级**: 全程用 1px 细线分隔，阴影仅在卡片悬浮时出现且极淡
3. **留白比深色模式更重要**: 白色有视觉膨胀感，需要更多呼吸空间（40px+ 页面边距，24px+ 卡片内边距）
4. **弥散光晕要淡**: blur 140px，opacity 不超过 0.35，颜色用淡蓝紫粉，避免刺眼
5. **代码块要轻**: 不用深色代码块，用浅灰底 + 蓝色关键字，保持整体清透
6. **按钮要果断**: Primary 按钮用纯黑底白字，不用渐变，保持极简
7. **聚焦态用蓝色 ring**: `0 0 0 3px rgba(37, 99, 235, 0.08)`，8% 透明度蓝色，既明显又不突兀

---

## 十、快速自检清单

复现完成后，检查以下项目：

- [ ] 背景是纯白 `#ffffff`，不是任何偏色
- [ ] 所有边框都是 `rgba(0,0,0,0.06)`，1px
- [ ] 没有使用任何深色模式元素
- [ ] 没有使用霓虹/发光效果（除极淡的球体阴影外）
- [ ] 没有使用厚重阴影（最大 `rgba(0,0,0,0.08)`）
- [ ] 文字层级清晰：黑 → 灰 → 浅灰
- [ ] 弥散光晕可见但不抢眼
- [ ] 网格纹理中心清晰、边缘淡出
- [ ] 三端布局符合各自屏幕尺寸约束
- [ ] 所有动画时长不超过 0.4s（除背景漂移外）
