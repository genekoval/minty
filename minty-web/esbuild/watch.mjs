import options from './options.mjs';

import * as esbuild from 'esbuild';

let ctx = await esbuild.context(options);

await ctx.watch();
