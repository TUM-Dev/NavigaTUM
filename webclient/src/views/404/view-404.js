// Alread pre-request root data:
navigatum.getExtendedData('root');

navigatum.registerView('404', {
    name: 'view-404',
    template: { gulp_inject: 'view-404.inc' },
    data: {},
});
