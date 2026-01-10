import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Kerminal',
  description: 'Modern Terminal Emulator & SSH Manager',
  base: '/kerminal/',

  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    ['link', { rel: 'icon', type: 'image/png', href: '/logo.png' }]
  ],

  locales: {
    root: {
      label: 'English',
      lang: 'en',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/' },
          { text: 'Guide', link: '/guide/getting-started' },
          { text: 'Features', link: '/guide/features' },
          { text: 'Download', link: 'https://github.com/klpod221/kerminal/releases/latest' }
        ],
        sidebar: {
          '/guide/': [
            {
              text: 'Introduction',
              items: [
                { text: 'Getting Started', link: '/guide/getting-started' },
                { text: 'Installation', link: '/guide/installation' },
                { text: 'Features', link: '/guide/features' }
              ]
            },
            {
              text: 'Terminal',
              items: [
                { text: 'Terminal Basics', link: '/guide/terminal-basics' },
                { text: 'Keyboard Shortcuts', link: '/guide/keyboard-shortcuts' }
              ]
            },
            {
              text: 'SSH & Remote',
              items: [
                { text: 'SSH Management', link: '/guide/ssh-management' },
                { text: 'SFTP File Browser', link: '/guide/sftp' }
              ]
            },
            {
              text: 'Productivity',
              items: [
                { text: 'Saved Commands', link: '/guide/saved-commands' },
                { text: 'Session Recording', link: '/guide/session-recording' }
              ]
            },
            {
              text: 'Data & Security',
              items: [
                { text: 'Sync & Security', link: '/guide/sync-security' }
              ]
            },
            {
              text: 'Development',
              items: [
                { text: 'Development Guide', link: '/guide/development' }
              ]
            }
          ]
        }
      }
    },
    vi: {
      label: 'Tiếng Việt',
      lang: 'vi',
      link: '/vi/',
      themeConfig: {
        nav: [
          { text: 'Trang chủ', link: '/vi/' },
          { text: 'Hướng dẫn', link: '/vi/guide/getting-started' },
          { text: 'Tính năng', link: '/vi/guide/features' },
          { text: 'Tải về', link: 'https://github.com/klpod221/kerminal/releases/latest' }
        ],
        sidebar: {
          '/vi/guide/': [
            {
              text: 'Giới thiệu',
              items: [
                { text: 'Bắt đầu', link: '/vi/guide/getting-started' },
                { text: 'Cài đặt', link: '/vi/guide/installation' },
                { text: 'Tính năng', link: '/vi/guide/features' }
              ]
            },
            {
              text: 'Terminal',
              items: [
                { text: 'Cơ bản Terminal', link: '/vi/guide/terminal-basics' },
                { text: 'Phím tắt', link: '/vi/guide/keyboard-shortcuts' }
              ]
            },
            {
              text: 'SSH & Remote',
              items: [
                { text: 'Quản lý SSH', link: '/vi/guide/ssh-management' },
                { text: 'Trình duyệt SFTP', link: '/vi/guide/sftp' }
              ]
            },
            {
              text: 'Năng suất',
              items: [
                { text: 'Lệnh đã lưu', link: '/vi/guide/saved-commands' },
                { text: 'Ghi phiên', link: '/vi/guide/session-recording' }
              ]
            },
            {
              text: 'Dữ liệu & Bảo mật',
              items: [
                { text: 'Đồng bộ & Bảo mật', link: '/vi/guide/sync-security' }
              ]
            },
            {
              text: 'Phát triển',
              items: [
                { text: 'Hướng dẫn phát triển', link: '/vi/guide/development' }
              ]
            }
          ]
        }
      }
    }
  },

  themeConfig: {
    logo: '/logo.png',

    socialLinks: [
      { icon: 'github', link: 'https://github.com/klpod221/kerminal' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2025 klpod221'
    },

    search: {
      provider: 'local'
    }
  }
})
