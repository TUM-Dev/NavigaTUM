// Alread pre-request root data:
navigatum.getExtendedData('root');

navigatum.registerView('main', {
    name: 'view-main',
    template: { gulp_inject: 'view-main.inc' },
    data: function() {
        return {
            root_data: null,
        };
    },
    beforeRouteEnter: function(to, from, next) {
        navigatum.getExtendedData('root').then((data) => next((vm) => vm.setData(data)));
    },
    beforeRouteUpdate: function(to, from, next) {
        // beforeRouteUpdate not used for now since data rarely changes
        next();
    },
    methods: {
        setData: function(data) {
            this.root_data = data;
            if (data !== null) navigatum.setTitle(data.name);
        },
        more: function(id) {
            document.getElementById(`panel-${id}`).classList.add('open');
        },
        less: function(id) {
            document.getElementById(`panel-${id}`).classList.remove('open');
        },
    },
});
