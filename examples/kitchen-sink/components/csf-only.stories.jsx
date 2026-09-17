import { CsfOnly } from './csf-only.jsx';

export default {
  title: 'Metadata/CSF only',
  description: 'This story is defined entirely in CSF. There is no matching TOML story entry.',
  component: CsfOnly,
  args: {
    label: 'Discovered from CSF',
    emphasis: 'quiet',
  },
  argTypes: {
    label: { control: 'text', name: 'Label', description: 'Visible card label supplied by CSF fallback metadata.' },
    emphasis: {
      control: 'select',
      name: 'Emphasis',
      description: 'Visual strength supplied by CSF fallback metadata.',
      options: ['quiet', 'strong'],
    },
  },
};

export const Default = {};
