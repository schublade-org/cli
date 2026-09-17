/** Mock /api/bootstrap payload. Same shape as live serve, not a second story format. */

const A11Y_RULES = [
  { id: "image-alt", label: "Images must expose an alt attribute" },
  { id: "button-name", label: "Buttons must have an accessible name" },
  { id: "link-name", label: "Links must have an accessible name" },
  { id: "label", label: "Form fields must be labelled" },
  { id: "control-name", label: "Controls must have an accessible name" },
];

const BUTTON_CONTROLS = [
  { kind: "text", id: "label", label: "Label", default: "Save changes" },
  {
    kind: "select",
    id: "variant",
    label: "Variant",
    default: "primary",
    options: [
      { value: "primary", label: "Primary" },
      { value: "secondary", label: "Secondary" },
      { value: "ghost", label: "Ghost" },
    ],
  },
  {
    kind: "select",
    id: "size",
    label: "Size",
    default: "md",
    options: [
      { value: "sm", label: "Small" },
      { value: "md", label: "Medium" },
      { value: "lg", label: "Large" },
    ],
  },
  { kind: "boolean", id: "disabled", label: "Disabled", default: false },
];

const BUTTON_TEMPLATE = `
<button class="btn" data-variant="{{variant}}" data-size="{{size}}" type="button"{{disabledAttr}}>{{label}}</button>
`;
const BUTTON_CODE =
  '<Button variant="{{variant}}" size="{{size}}" disabled={{{disabled}}}>{{label}}</Button>';

export const bootstrap = {
  catalog: {
    name: "Aarau Designsystem",
    stories: [
      {
        id: "accordion",
        title: "Accordion",
        group: null,
        item: "Accordion",
        section: "Components",
        description: "Reveals a section of supporting content without leaving the page.",
        generator: "html",
        code: '<Accordion title="{{title}}" defaultOpen={{{open}}} />',
        template: `
<div class="acc">
  <details class="acc-item"{{openAttr}}>
    <summary class="acc-summary">{{title}}</summary>
    <div class="acc-body">{{body}}</div>
  </details>
</div>
`,
        controls: [
          { kind: "text", id: "title", label: "Title", default: "Delivery and returns" },
          {
            kind: "text",
            id: "body",
            label: "Body",
            default: "Orders placed before 14:00 CET leave Aarau the same day.",
          },
          { kind: "boolean", id: "open", label: "Open by default", default: true },
        ],
      },
      {
        id: "avatar-group",
        title: "Avatar group",
        group: null,
        item: "Avatar group",
        section: "Components",
        description: "Summarizes several people as an overlapping, accessible avatar stack.",
        generator: "html",
        code: '<AvatarGroup aria-label="{{total}} members" max={{{visible}}} size="{{size}}" />',
        template: `
<div class="ag" data-size="{{size}}" role="img" aria-label="{{total}} members">
  <span class="ag-face">AN</span>
  <span class="ag-face">MK</span>
  <span class="ag-face">+{{visible}}</span>
</div>
`,
        controls: [
          {
            kind: "select",
            id: "size",
            label: "Size",
            default: "md",
            options: [
              { value: "sm", label: "Small" },
              { value: "md", label: "Medium" },
              { value: "lg", label: "Large" },
            ],
          },
          { kind: "number", id: "total", label: "Total avatars", min: 1, max: 12, default: 7 },
          {
            kind: "number",
            id: "visible",
            label: "Visible before overflow",
            min: 1,
            max: 8,
            default: 4,
          },
          { kind: "boolean", id: "showCount", label: "Show member count", default: true },
        ],
      },
      {
        id: "badge",
        title: "Badge",
        group: null,
        item: "Badge",
        section: "Components",
        description: "A compact status label for counts, states, and metadata.",
        generator: "html",
        code: '<Badge tone="{{tone}}">{{label}}</Badge>',
        template: '<span class="badge" data-tone="{{tone}}">{{label}}</span>',
        controls: [
          {
            kind: "select",
            id: "tone",
            label: "Tone",
            default: "neutral",
            options: [
              { value: "neutral", label: "Neutral" },
              { value: "accent", label: "Accent" },
              { value: "warning", label: "Warning" },
            ],
          },
          { kind: "text", id: "label", label: "Label", default: "In review" },
        ],
      },
      {
        id: "button",
        title: "Button / Default",
        group: "Button",
        item: "Default",
        section: "Components",
        description: "The primary action control. Keep the label short and specific.",
        generator: "html",
        code: BUTTON_CODE,
        template: BUTTON_TEMPLATE,
        controls: BUTTON_CONTROLS,
      },
      {
        id: "button-ghost",
        title: "Button / Ghost",
        group: "Button",
        item: "Ghost",
        section: "Components",
        description: "A quiet action that sits next to a primary button.",
        generator: "html",
        code: BUTTON_CODE,
        template: BUTTON_TEMPLATE,
        controls: BUTTON_CONTROLS.map((control) =>
          control.id === "label"
            ? { ...control, default: "Cancel" }
            : control.id === "variant"
              ? { ...control, default: "ghost" }
              : control
        ),
      },
      {
        id: "button-disabled",
        title: "Button / Disabled",
        group: "Button",
        item: "Disabled",
        section: "Components",
        description: "A primary action that cannot be activated.",
        generator: "html",
        code: BUTTON_CODE,
        template: BUTTON_TEMPLATE,
        controls: BUTTON_CONTROLS.map((control) =>
          control.id === "disabled" ? { ...control, default: true } : control
        ),
      },
      {
        id: "chip",
        title: "Chip",
        group: null,
        item: "Chip",
        section: "Components",
        description: "A compact choice or filter token, optionally dismissible.",
        generator: "html",
        code: '<Chip selected={{{selected}}} dismissible={{{dismissible}}}>{{label}}</Chip>',
        template: `
<span class="chip" data-selected="{{selected}}">
  <span>{{label}}</span>
  <button type="button" class="chip-x" aria-label="Remove {{label}}"{{dismissAttr}}>×</button>
</span>
`,
        controls: [
          { kind: "text", id: "label", label: "Label", default: "Design system" },
          { kind: "boolean", id: "selected", label: "Selected", default: true },
          { kind: "boolean", id: "dismissible", label: "Dismissible", default: true },
        ],
      },
      {
        id: "mark",
        title: "Mark",
        group: null,
        item: "Mark",
        section: "Foundations",
        description: "A static wordmark with no controls.",
        generator: "html",
        code: "<Mark />",
        template: '<span class="badge" data-tone="neutral">Schublade</span>',
        controls: [],
      },
      {
        id: "missing-name",
        title: "Missing name",
        group: null,
        item: "Missing name",
        section: "Foundations",
        description: "Intentionally unnamed control so the a11y inspector has something to show.",
        generator: "html",
        code: "<IconButton />",
        template: `
<button type="button" class="btn" data-variant="ghost" data-size="md"></button>
<img src="data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==" width="16" height="16" />
`,
        controls: [],
      },
    ],
  },
  theme: {
    trigger: "data-attribute",
    key: "data-theme",
    light: "light",
    dark: "dark",
  },
  a11y: {
    enabled: true,
    rules: A11Y_RULES,
  },
  brand: {
    name: "Aarau Designsystem",
    logo: "/brand/logo",
    favicon: "/brand/favicon",
  },
};
