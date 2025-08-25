
import type { NavItem, SocialLink } from "$lib/types/nav";

import {
    Boxes,
    Paintbrush,
    Workflow,
    Zap
} from 'lucide-svelte';
import type { Feature, PromoConfig, SiteConfig } from "./types/config";


export const siteConfig: SiteConfig = {
    version: '0.174.28875',
    title: 'BoxMux Documentation',
    description:
        'Documentation for BoxMux - A YAML-driven terminal UI framework for CLI applications and dashboards.',
    github: 'https://github.com/jowharshamshiri/boxmux',
    npm: 'boxmux',

    quickLinks: [
        { title: 'Configuration', href: '/docs/configuration' },
        { title: 'User Guide', href: '/docs/user-guide' },
        { title: 'API Reference', href: '/docs/api' },
        { title: 'PTY Features', href: '/docs/pty-features' }
    ],
    logo: '/logo.svg',
    logoDark: '/logo-white.svg',
    favicon: '/favicon.png',
};


export let navItems: NavItem[] = [
    {
        title: 'Docs',
        href: '/docs'
    },

];

export let socialLinks: SocialLink[] = [
    {
        title: 'GitHub',
        href: 'https://github.com/jowharshamshiri/boxmux',
        icon: 'github'
    },
];


export const features: Feature[] = [
    {
        icon: Boxes,
        title: 'YAML Configuration',
        description: 'Define terminal interfaces using YAML files with nested box layouts, menus, and content'
    },
    {
        icon: Workflow,
        title: 'Script Execution',
        description: 'Execute shell scripts with output streaming, background threading, and output redirection'
    },
    {
        icon: Paintbrush,
        title: 'PTY Support',
        description: 'Pseudo-terminal integration for running interactive programs like vim, htop, and SSH'
    },
    {
        icon: Zap,
        title: 'Socket Remote Control',
        description: 'Control BoxMux applications via Unix sockets with CLI and API commands'
    }
];

export let promoConfig: PromoConfig = {
    title: 'Build terminal UIs with YAML',
    description:
        'BoxMux uses YAML configuration files to define CLI applications with real-time execution.',
    ctaText: "Documentation",
    ctaLink: '/docs',
    lightImage: '/screenshot.png',
    darkImage: '/screenshot.png'
};