import { HoverContent } from '@/components/hover-content';
import { hoverTooltip } from '@codemirror/view';
import { createRoot } from 'react-dom/client';

import { hoverSource } from '../analyzer';

export const hoverExtension = hoverTooltip(
  async (view, position, side) => {
    const { doc } = view.state;
    const offset = position + (side < 0 ? -1 : 0);

    if (offset < 0 || offset >= doc.length) return null;

    const line = doc.lineAt(offset);
    const hover = await hoverSource(
      doc.toString(),
      line.number - 1,
      offset - line.from
    );

    if (!hover || view.state.doc !== doc) return null;

    const from = doc.line(hover.startLine + 1).from + hover.startCharacter;
    const to = doc.line(hover.endLine + 1).from + hover.endCharacter;

    if (offset < from || offset >= to) return null;

    return {
      pos: from,
      end: to,
      above: true,
      create: () => {
        const dom = document.createElement('div');
        dom.className =
          'max-h-80 max-w-[min(36rem,calc(100vw-2rem))] overflow-auto p-3';

        const root = createRoot(dom);
        root.render(<HoverContent {...hover} />);

        return { dom, destroy: () => root.unmount() };
      },
    };
  },
  { hideOnChange: true }
);
