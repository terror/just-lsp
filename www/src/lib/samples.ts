import attributes from '../../samples/attributes.just?raw';
import dependencies from '../../samples/dependencies.just?raw';
import diagnostics from '../../samples/diagnostics.just?raw';
import parameters from '../../samples/parameters.just?raw';
import scripts from '../../samples/scripts.just?raw';
import variables from '../../samples/variables.just?raw';

export const samples = [
  { name: 'attributes', code: attributes },
  { name: 'dependencies', code: dependencies },
  { name: 'diagnostics', code: diagnostics },
  { name: 'parameters', code: parameters },
  { name: 'scripts', code: scripts },
  { name: 'variables', code: variables },
].map((sample) => ({ ...sample, code: sample.code.trim() }));

export function getSample(name: string) {
  return samples.find((sample) => sample.name === name) ?? samples[0];
}
