import { diffLines } from 'diff';
import { type Root } from 'mdast';

export const remarkExampleDiffs = () => (tree: Root) => {
  for (let index = 0; index < tree.children.length - 1; index++) {
    const reported = tree.children[index];
    const corrected = tree.children[index + 1];

    if (
      reported.type !== 'code' ||
      corrected.type !== 'code' ||
      reported.lang !== 'just' ||
      corrected.lang !== 'just' ||
      reported.meta !== 'reported' ||
      corrected.meta !== 'corrected'
    ) {
      continue;
    }

    tree.children.splice(index, 2, {
      type: 'code',
      lang: 'diff',
      meta: 'reported-corrected',
      value: diffLines(
        reported.value ? `${reported.value}\n` : '',
        corrected.value ? `${corrected.value}\n` : ''
      )
        .flatMap(({ added, removed, value }) =>
          value
            .replace(/\n$/, '')
            .split('\n')
            .map((line) => `${removed ? '-' : added ? '+' : ' '}${line}`)
        )
        .join('\n'),
    });
  }
};
