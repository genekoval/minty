export default {
    loader: { '.ttf': 'file' },
    entryPoints: ['index.js'],
    assetNames: '[dir]/[name]',
    bundle: true,
    outdir: '../crates/mintyd/assets',
    publicPath: '/assets',
    logLevel: 'info',
};
