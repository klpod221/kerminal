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
                { text: 'Features', link: '/guide/features' },
                { text: 'Installation', link: '/guide/installation' }
              ]
            },
            {
              text: 'Usage',
              items: [
                { text: 'SSH Management', link: '/guide/ssh-management' },
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
                { text: 'Tính năng', link: '/vi/guide/features' },
                { text: 'Cài đặt', link: '/vi/guide/installation' }
              ]
            },
            {
              text: 'Sử dụng',
              items: [
                { text: 'Quản lý SSH', link: '/vi/guide/ssh-management' },
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
