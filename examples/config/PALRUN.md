# PALRUN.md - Project Rules Example

This file is read by Palrun's AI when you're working in this project.
Place it in your project root as `PALRUN.md` or `.palrun/agent.md`.

## Project Overview

Describe your project here so the AI understands the context:

- **Project Name**: My Awesome App
- **Tech Stack**: Node.js, TypeScript, React, PostgreSQL
- **Build System**: npm

## Coding Standards

### Style Guide

- Use TypeScript strict mode
- Prefer `const` over `let`
- Use async/await over raw promises
- Use functional components with hooks in React

### File Organization

```
src/
  components/   # React components
  hooks/        # Custom hooks
  services/     # API services
  utils/        # Utility functions
  types/        # TypeScript types
```

## Common Commands

The AI can help you run these commands:

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm test` - Run tests
- `npm run lint` - Check code quality

## Dependencies

Key dependencies to be aware of:

- React 18 with concurrent features
- TanStack Query for data fetching
- Zod for validation
- Tailwind CSS for styling

## AI Behavior Guidelines

When using AI in this project:

1. **Testing**: Always suggest running tests after code changes
2. **Types**: Prefer strict TypeScript types, avoid `any`
3. **Commits**: Use conventional commits format (feat:, fix:, etc.)
4. **Security**: Never commit secrets, use environment variables

## Environment Variables

Required environment variables (add to `.env.local`):

```env
DATABASE_URL=postgresql://...
API_KEY=your-api-key
```

## Additional Notes

Add any project-specific notes here that would help the AI assist you better.
