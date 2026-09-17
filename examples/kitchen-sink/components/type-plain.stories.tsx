import { TypePlain } from './type-plain.tsx';

export default {
  title: 'Props/Type alias - no comments',
  description: 'A TypeScript type alias with no prop comments. Types and runtime defaults still come from the AST.',
  component: TypePlain,
  args: {
    title: 'Type alias without comments',
    tone: 'neutral',
    count: 3,
    muted: false,
  },
  argTypes: {
    title: { control: 'text', name: 'Title' },
    tone: {
      control: 'select',
      name: 'Tone',
      options: [
        { value: 'neutral', label: 'Neutral' },
        { value: 'accent', label: 'Accent' },
        { value: 'positive', label: 'Positive' },
      ],
    },
    count: { control: 'number', name: 'Count', min: 0, max: 12 },
    muted: { control: 'boolean', name: 'Muted' },
  },
};

export const Default = {};
