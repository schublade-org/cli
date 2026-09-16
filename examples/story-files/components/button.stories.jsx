import { Button } from './button.jsx';

export default {
  title: 'Components/Button',
  description: 'The primary action control. Keep the label short and specific.',
  component: Button,
  args: {
    label: 'Save changes',
    variant: 'primary',
    size: 'md',
    disabled: false,
  },
  argTypes: {
    label: { control: 'text', name: 'Label' },
    variant: {
      control: 'select',
      name: 'Variant',
      options: [
        { value: 'primary', label: 'Primary' },
        { value: 'secondary', label: 'Secondary' },
        { value: 'ghost', label: 'Ghost' },
      ],
    },
    size: {
      control: 'select',
      name: 'Size',
      options: [
        { value: 'sm', label: 'Small' },
        { value: 'md', label: 'Medium' },
        { value: 'lg', label: 'Large' },
      ],
    },
    disabled: { control: 'boolean', name: 'Disabled' },
  },
};

export const Default = {};

export const Ghost = {
  args: { variant: 'ghost', label: 'Cancel' },
};

export const Disabled = {
  args: { disabled: true },
};
