import { InterfaceCommented } from './interface-commented.tsx';

export default {
  title: 'Props/Interface - comments',
  description: 'Every interface member is documented in source and appears in the generated Props table.',
  component: InterfaceCommented,
  args: {
    label: 'Documented interface',
    density: 'compact',
    progress: 72,
    complete: true,
  },
  argTypes: {
    label: { control: 'text', name: 'Label' },
    density: { control: 'select', name: 'Density', options: ['comfortable', 'compact'] },
    progress: { control: 'number', name: 'Progress', min: 0, max: 100 },
    complete: { control: 'boolean', name: 'Complete' },
  },
};

export const Default = {};
