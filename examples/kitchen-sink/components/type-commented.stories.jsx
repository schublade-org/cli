import { TypeCommented } from './type-commented.tsx';

export default {
  title: 'Props/Type alias - comments',
  description: 'Every declared prop has a source comment, including a declared-only prop that is not destructured.',
  component: TypeCommented,
  args: {
    title: 'Documented type alias',
    tone: 'accent',
    count: 6,
    muted: false,
  },
  argTypes: {
    title: { control: 'text', name: 'Title' },
    tone: {
      control: 'select',
      name: 'Tone',
      options: ['neutral', 'accent', 'positive'],
    },
    count: { control: 'number', name: 'Count', min: 0, max: 12 },
    muted: { control: 'boolean', name: 'Muted' },
  },
};

export const Default = {};
