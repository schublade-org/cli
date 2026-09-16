import html from './badge.html';

export default {
  title: 'Components/Badge',
  description: 'A compact status label for counts, states, and metadata.',
  component: html,
  args: {
    tone: 'neutral',
    label: 'In review',
  },
  argTypes: {
    tone: {
      control: 'select',
      name: 'Tone',
      options: [
        { value: 'neutral', label: 'Neutral' },
        { value: 'accent', label: 'Accent' },
        { value: 'warning', label: 'Warning' },
      ],
    },
    label: { control: 'text', name: 'Label' },
  },
};
