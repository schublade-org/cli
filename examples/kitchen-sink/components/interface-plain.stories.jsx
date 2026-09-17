import { InterfacePlain } from './interface-plain.tsx';

export default {
  title: 'Props/Interface - no comments',
  description: 'A TypeScript interface without prop comments. The Props table still shows exact declared types and defaults.',
  component: InterfacePlain,
  args: {
    label: 'Interface without comments',
    density: 'comfortable',
    progress: 40,
    complete: false,
  },
  argTypes: {
    label: { control: 'text', name: 'Label' },
    density: { control: 'select', name: 'Density', options: ['comfortable', 'compact'] },
    progress: { control: 'number', name: 'Progress', min: 0, max: 100 },
    complete: { control: 'boolean', name: 'Complete' },
  },
};

export const Default = {};
