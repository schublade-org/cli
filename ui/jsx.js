(function (global) {
  function transformJsx(source) {
    return new Scanner(source).transform();
  }

  function rewriteExports(source) {
    const extras = [];
    let out = source
      .replace(/import\s+[^;]+;?/g, "")
      .replace(/export\s+default\s+function\s+(\w+)/g, (_, name) => {
        extras.push(`exports.default = ${name}; exports[${JSON.stringify(name)}] = ${name};`);
        return `function ${name}`;
      })
      .replace(/export\s+function\s+(\w+)/g, (_, name) => {
        extras.push(`exports[${JSON.stringify(name)}] = ${name};`);
        return `function ${name}`;
      })
      .replace(/export\s+const\s+(\w+)\s*=/g, (_, name) => {
        extras.push(`exports[${JSON.stringify(name)}] = ${name};`);
        return `const ${name} =`;
      })
      .replace(/export\s+default\s+/g, "exports.default = ");
    if (extras.length) {
      out += `\n${extras.join("\n")}\n`;
    }
    return out;
  }

  function evaluateModule(source, exportName) {
    const exports = {};
    const module = { exports };
    const transformed = transformJsx(rewriteExports(source));
    const factory = new Function(
      "React",
      "exports",
      "module",
      `${transformed}\nreturn exports[${JSON.stringify(exportName)}] || exports.default || module.exports.${exportName} || module.exports.default || module.exports;`
    );
    const Component = factory(global.React, exports, module);
    if (typeof Component !== "function") {
      throw new Error(`Could not evaluate export "${exportName}"`);
    }
    return Component;
  }

  function Scanner(src) {
    this.src = src;
    this.i = 0;
  }

  Scanner.prototype.transform = function () {
    return this.copyJs(this.src.length);
  };

  Scanner.prototype.copyJs = function (end) {
    let out = "";
    while (this.i < end) {
      const start = this.i;
      if (this.startsJsx()) {
        out += this.parseElement();
        continue;
      }
      const ch = this.src[this.i];
      if (ch === '"' || ch === "'" || ch === "`") {
        out += this.readString();
        continue;
      }
      if (ch === "/" && this.src[this.i + 1] === "/") {
        const nl = this.src.indexOf("\n", this.i);
        this.i = nl === -1 ? this.src.length : nl + 1;
        out += this.src.slice(start, this.i);
        continue;
      }
      if (ch === "/" && this.src[this.i + 1] === "*") {
        const close = this.src.indexOf("*/", this.i + 2);
        this.i = close === -1 ? this.src.length : close + 2;
        out += this.src.slice(start, this.i);
        continue;
      }
      out += ch;
      this.i += 1;
    }
    return out;
  };

  Scanner.prototype.startsJsx = function () {
    if (this.src[this.i] !== "<") {
      return false;
    }
    const next = this.src[this.i + 1];
    return next === "/" || next === ">" || isIdentStart(next);
  };

  Scanner.prototype.parseElement = function () {
    this.expect("<");
    if (this.src[this.i] === "/") {
      throw new Error("unexpected closing JSX tag");
    }
    if (this.src[this.i] === ">") {
      this.i += 1;
      const children = this.parseChildren();
      return `React.createElement(React.Fragment, null${children})`;
    }
    const name = this.readTagName();
    const typeExpr = isComponentTag(name) ? name : JSON.stringify(name);
    const props = this.parseProps();
    this.skipWs();
    if (this.src[this.i] === "/" && this.src[this.i + 1] === ">") {
      this.i += 2;
      return `React.createElement(${typeExpr}, ${props})`;
    }
    this.expect(">");
    const children = this.parseChildren();
    return `React.createElement(${typeExpr}, ${props}${children})`;
  };

  Scanner.prototype.parseProps = function () {
    const parts = [];
    const spreads = [];
    while (this.i < this.src.length) {
      this.skipWs();
      if (this.src[this.i] === ">" || (this.src[this.i] === "/" && this.src[this.i + 1] === ">")) {
        break;
      }
      if (this.src.startsWith("{...", this.i)) {
        this.i += 1;
        const innerEnd = this.scanBalanced("{", "}", this.i);
        const expr = this.src.slice(this.i, innerEnd).replace(/^\.\.\./, "");
        this.i = innerEnd + 1;
        spreads.push(new Scanner(expr).transform());
        continue;
      }
      const key = this.readIdent();
      if (!key) {
        break;
      }
      this.skipWs();
      if (this.src[this.i] === "=") {
        this.i += 1;
        this.skipWs();
        if (this.src[this.i] === "{") {
          const expr = this.readBalanced("{", "}");
          parts.push(`${JSON.stringify(key)}: ${expr}`);
        } else if (this.src[this.i] === '"' || this.src[this.i] === "'") {
          parts.push(`${JSON.stringify(key)}: ${this.readString()}`);
        } else {
          throw new Error(`unsupported value for JSX prop ${key}`);
        }
      } else {
        parts.push(`${JSON.stringify(key)}: true`);
      }
    }
    if (!parts.length && !spreads.length) {
      return "null";
    }
    const object = `{${parts.join(", ")}}`;
    if (!spreads.length) {
      return object;
    }
    return `Object.assign({}, ${object}${spreads.map((item) => `, ${item}`).join("")})`;
  };

  Scanner.prototype.parseChildren = function () {
    const children = [];
    while (this.i < this.src.length) {
      if (this.src[this.i] === "<" && this.src[this.i + 1] === "/") {
        this.i += 2;
        this.readTagName();
        this.skipWs();
        this.expect(">");
        break;
      }
      if (this.src[this.i] === "{") {
        const expr = this.readBalanced("{", "}");
        if (expr.trim()) {
          children.push(expr);
        }
        continue;
      }
      if (this.startsJsx()) {
        children.push(this.parseElement());
        continue;
      }
      const textStart = this.i;
      while (this.i < this.src.length && this.src[this.i] !== "<" && this.src[this.i] !== "{") {
        this.i += 1;
      }
      const raw = this.src.slice(textStart, this.i);
      if (shouldKeepText(raw)) {
        children.push(JSON.stringify(collapseText(raw)));
      }
    }
    return children.length ? `, ${children.join(", ")}` : "";
  };

  Scanner.prototype.readBalanced = function (open, close) {
    this.expect(open);
    const innerStart = this.i;
    const innerEnd = this.scanBalanced(open, close, innerStart);
    const inner = this.src.slice(innerStart, innerEnd);
    const transformed = new Scanner(inner).transform();
    this.i = innerEnd + 1;
    return transformed;
  };

  Scanner.prototype.scanBalanced = function (open, close, from) {
    let depth = 1;
    let i = from;
    while (i < this.src.length) {
      const ch = this.src[i];
      if (ch === '"' || ch === "'" || ch === "`") {
        i = skipString(this.src, i);
        continue;
      }
      if (ch === "/" && this.src[i + 1] === "/") {
        const nl = this.src.indexOf("\n", i);
        i = nl === -1 ? this.src.length : nl + 1;
        continue;
      }
      if (ch === "/" && this.src[i + 1] === "*") {
        const closeIdx = this.src.indexOf("*/", i + 2);
        i = closeIdx === -1 ? this.src.length : closeIdx + 2;
        continue;
      }
      if (ch === open) {
        depth += 1;
      } else if (ch === close) {
        depth -= 1;
        if (depth === 0) {
          return i;
        }
      }
      i += 1;
    }
    throw new Error(`unclosed ${open}${close}`);
  };

  Scanner.prototype.readString = function () {
    const start = this.i;
    this.i = skipString(this.src, this.i);
    return this.src.slice(start, this.i);
  };

  Scanner.prototype.readTagName = function () {
    const start = this.i;
    while (this.i < this.src.length && /[A-Za-z0-9_$.-]/.test(this.src[this.i])) {
      this.i += 1;
    }
    return this.src.slice(start, this.i);
  };

  Scanner.prototype.readIdent = function () {
    if (!isIdentStart(this.src[this.i])) {
      return "";
    }
    const start = this.i;
    this.i += 1;
    while (this.i < this.src.length && /[A-Za-z0-9_$-]/.test(this.src[this.i])) {
      this.i += 1;
    }
    return this.src.slice(start, this.i);
  };

  Scanner.prototype.skipWs = function () {
    while (this.i < this.src.length && /\s/.test(this.src[this.i])) {
      this.i += 1;
    }
  };

  Scanner.prototype.expect = function (ch) {
    if (this.src[this.i] !== ch) {
      throw new Error(`expected "${ch}"`);
    }
    this.i += 1;
  };

  function skipString(src, i) {
    const quote = src[i];
    i += 1;
    while (i < src.length) {
      if (src[i] === "\\") {
        i += 2;
        continue;
      }
      if (src[i] === quote) {
        return i + 1;
      }
      if (quote === "`" && src[i] === "$" && src[i + 1] === "{") {
        i += 2;
        let depth = 1;
        while (i < src.length && depth) {
          if (src[i] === '"' || src[i] === "'" || src[i] === "`") {
            i = skipString(src, i);
            continue;
          }
          if (src[i] === "{") depth += 1;
          else if (src[i] === "}") depth -= 1;
          i += 1;
        }
        continue;
      }
      i += 1;
    }
    return src.length;
  }

  function isIdentStart(ch) {
    return typeof ch === "string" && /[A-Za-z_$]/.test(ch);
  }

  function isComponentTag(name) {
    return /^[A-Z]/.test(name) || name.includes(".");
  }

  function shouldKeepText(raw) {
    if (!raw) return false;
    if (/^\s*$/.test(raw) && raw.includes("\n")) return false;
    return !/^\s*$/.test(raw);
  }

  function collapseText(raw) {
    return raw.replace(/\s+/g, " ");
  }

  global.transformJsx = transformJsx;
  global.evaluateJsxModule = evaluateModule;
})(window);
