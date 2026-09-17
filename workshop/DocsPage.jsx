import { ColorScales } from "./ColorScales.jsx";
import { TypographyDocs } from "./TypographyDocs.jsx";
import { filterBySource } from "./tokens.js";

export function DocsPage({ page, tokens }) {
  const colors = tokens?.colors ?? [];
  const typography = tokens?.typography ?? { styles: [], families: [], roles: [] };

  return (
    <div className="docs">
      {page.blocks.map((block, index) => {
        switch (block.type) {
          case "heading":
            return (
              <Heading key={index} level={block.level}>
                {block.text}
              </Heading>
            );
          case "paragraph":
            return (
              <p className="docs-copy" key={index}>
                {block.text}
              </p>
            );
          case "color-scales":
            return (
              <ColorScales
                key={index}
                scales={block.scales ?? filterBySource(colors, block.source)}
              />
            );
          case "color-scale":
            return <ColorScales key={index} scales={block.scale ? [block.scale] : []} />;
          case "typography":
            return (
              <TypographyDocs
                key={index}
                typography={{
                  styles: filterBySource(typography.styles || [], block.source),
                  families: filterBySource(typography.families || [], block.source),
                  roles: filterBySource(typography.roles || [], block.source),
                }}
              />
            );
          default: {
            const _never = block.type;
            void _never;
            return null;
          }
        }
      })}
    </div>
  );
}

function Heading({ level, children }) {
  switch (level) {
    case 1:
      return <h1 className="docs-heading">{children}</h1>;
    case 2:
      return <h2 className="docs-heading">{children}</h2>;
    case 3:
      return <h3 className="docs-heading">{children}</h3>;
    case 4:
      return <h4 className="docs-heading">{children}</h4>;
    case 5:
      return <h5 className="docs-heading">{children}</h5>;
    default:
      return <h6 className="docs-heading">{children}</h6>;
  }
}
