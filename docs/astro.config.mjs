import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://spidermedic.com',
  integrations: [
    starlight({
      title: 'spidermedic',
      description: 'Crawl a website and validate HTTP responses — catch broken links in CI before they reach production.',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/ethanhann/spidermedic' },
      ],
      sidebar: [
        {
          label: 'Guides',
          items: [
            { label: 'GitHub Action', link: '/guides/action' },
            { label: 'CLI', link: '/guides/cli' },
          ],
        },
      ],
    }),
  ],
});
