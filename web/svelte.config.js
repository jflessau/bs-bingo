import sveltePreprocess from 'svelte-preprocess';
import * as sass from 'sass';

export default {
  preprocess: sveltePreprocess({
    sass: {
      sync: true,
      implementation: sass,
    },
  }),
};
