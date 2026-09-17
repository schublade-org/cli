import { Tooltip } from "@base-ui/react/tooltip";

export function IconButton({
  label,
  onClick,
  disabled = false,
  active = false,
  className = "icon-btn",
  children,
}) {
  const classes = [className, active ? "is-active" : ""].filter(Boolean).join(" ");
  return (
    <Tooltip.Root>
      <Tooltip.Trigger
        className={classes}
        aria-label={label}
        disabled={disabled}
        onClick={onClick}
      >
        {children}
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Positioner sideOffset={6}>
          <Tooltip.Popup className="tooltip">{label}</Tooltip.Popup>
        </Tooltip.Positioner>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}
