// To work even when the rest of the JS code failed, the code for the
// feedback form is mostly seperate from the rest of the codebase.
// It is only loaded when the feedback form is being opened.

var feedback = (function () {
    var token = null;

    function do_request(method, url, data, onsuccess, onerror) {
        var req = new XMLHttpRequest();
        req.open(method, window.encodeURI(url), true);
        req.onload = function () {
            onsuccess(this);
        };
        req.onerror = function () {
            onerror(this);
        };
        if (data === null) {
            req.send();
        } else {
            req.setRequestHeader('Content-Type', 'application/json');
            req.send(data);
        }
    }

    function show_error(msg, block_send) {
        msg = msg || '';
        document.getElementById('feedback-error').innerText = msg;
        document.getElementById('feedback-send').disabled = block_send || false;
    }

    function show_loading(is_loading) {
        if (is_loading) {
            document.getElementById('feedback-send').classList.add('loading');
            document.getElementById('feedback-send').disabled = true;
        } else {
            document.getElementById('feedback-send').classList.remove('loading');
            document.getElementById('feedback-send').disabled = false;
        }
    }

    function open_form(category, subject, body) {
        category = category || 'general';
        subject = subject || '';
        body = body || '';

        document.getElementById('feedback-category').value = category;
        document.getElementById('feedback-subject').value = subject;
        document.getElementById('feedback-body').value = body;
        document.getElementById('feedback-privacy').checked = false;
        document.getElementById('feedback-delete').checked = false;

        show_error(false);
        show_loading(false);

        document.getElementById('feedback-modal').classList.add('active');
        document.body.classList.add('no-scroll');

        // Token are renewed after 6 hours here to be sure, even though they may be valid
        // for longer on the server side.
        if (token === null && navigatum) {
            token = navigatum.getLocalStorageWithExpiry('feedback-token', null);
        }
        if (token === null || Date.now() - token.creation > 1000 * 3600 * 6) {
            do_request(
                'POST',
                '/* @echo api_prefix */feedback/get_token',
                null,
                function (r) {
                    if (r.status === 201) {
                        token = {
                            creation: Date.now(),
                            value: JSON.parse(r.response).token,
                        };
                        if (navigatum)
                            navigatum.setLocalStorageWithExpiry('feedback-token', token, 6);
                    } else if (r.status === 429) {
                        show_error('${{_.feedback.error.429}}$', true);
                    } else if (r.status === 503) {
                        show_error('${{_.feedback.error.503}}$', true);
                    } else {
                        const unexpectedTokenError =
                            '${{_.feedback.error.token_unexpected_status}}$';
                        show_error(`${unexpectedTokenError}${r.status}`, true);
                    }
                },
                function (r) {
                    show_error('${{_.feedback.error.token_req_failed}}$');
                    console.log(r);
                },
            );
        }
    }

    function update_feedback_form(category) {
        if (category === undefined) category = document.getElementById('feedback-category').value;

        const helptextLUT = {
            general: '${{_.feedback.helptext.general}}$',
            bug: '${{_.feedback.helptext.bug}}$',
            features: '${{_.feedback.helptext.features}}$',
            search: '${{_.feedback.helptext.search}}$',
            entry: '${{_.feedback.helptext.entry}}$',
        };
        const feedback_helptext = document.getElementById('feedback-helptext');
        feedback_helptext.innerText = helptextLUT[category];

        const coordinate_picker = document.getElementById('feedback-coordinate-picker');
        if (category === 'entry') {
            coordinate_picker.classList.remove('d-none');
        } else {
            coordinate_picker.classList.add('d-none');
        }
    }

    function close_form() {
        document.getElementById('feedback-coordinate-picker').classList.add('d-none');
        document.getElementById('feedback-coordinate-picker-helptext').classList.add('d-none');

        document.getElementById('feedback-modal').classList.remove('active');
        document.getElementById('feedback-success-modal').classList.remove('active');

        document.body.classList.remove('no-scroll');
    }

    function may_close_form() {
        if (document.getElementById('feedback-body').value.length == 0) close_form();
    }

    function send_form() {
        if (token === null) {
            show_error('${{_.feedback.error.send_no_token}}$', true);
        } else if (document.getElementById('feedback-subject').value.length < 3) {
            show_error('${{_.feedback.error.too_short_subject}}$');
        } else if (document.getElementById('feedback-body').value.length < 10) {
            show_error('${{_.feedback.error.too_short_body}}$');
        } else if (document.getElementById('feedback-privacy').checked !== true) {
            show_error('${{_.feedback.error.privacy_not_checked}}$');
        } else {
            show_loading(true);
            // Token may only be used after a short delay. In case that has not passed
            // yet, we wait until for a short time.
            if (Date.now() - token.creation < 1000 * 10) {
                window.setTimeout(send, 1000 * 10 - (Date.now() - token.creation));
            } else {
                send();
            }
        }
    }

    function show_success(href) {
        document.getElementById('feedback-modal').classList.remove('active');
        document.getElementById('feedback-success-modal').classList.add('active');
        document.getElementById('feedback-success-url').setAttribute('href', href);
    }

    function send() {
        var category = document.getElementById('feedback-category').value;
        var subject = document.getElementById('feedback-subject').value;
        var body = document.getElementById('feedback-body').value;
        var privacy = document.getElementById('feedback-privacy').checked;
        var delete_issue = document.getElementById('feedback-delete').checked;

        do_request(
            'POST',
            '/* @echo api_prefix */feedback/feedback',
            JSON.stringify({
                token: token.value,
                category: category,
                subject: subject,
                body: body,
                privacy_checked: privacy,
                delete_issue_requested: delete_issue,
            }),
            function (r) {
                show_loading(false);
                if (r.status === 201) {
                    localStorage.removeItem('coordinate-feedback');
                    token = null;
                    localStorage.removeItem('feedback-token');
                    var e = new Event('storage');
                    window.dispatchEvent(e);
                    show_success(r.responseText);
                } else if (r.status === 500) {
                    const serverError = '${{_.feedback.error.server_error}}$';
                    show_error(`${serverError} (${r.responseText})`, false);
                } else if (r.status === 451) {
                    show_error('${{_.feedback.error.privacy_not_checked}}$', false);
                } else if (r.status === 403) {
                    localStorage.removeItem('feedback-token');
                    token = null;
                    const invalidTokenError = '${{_.feedback.error.send_invalid_token}}$';
                    show_error(`${invalidTokenError} (${r.responseText})`, false);
                } else {
                    const unexpectedStatusError = '${{_.feedback.error.send_invalid_token}}$';
                    show_error(`${unexpectedStatusError}${r.status}`, false);
                }
            },
            function (r) {
                show_loading(false);
                show_error('${{_.feedback.error.send_req_failed}}$');
                console.log(r);
            },
        );
    }

    document.getElementById('feedback-cancel').addEventListener('click', close_form, false);
    document.getElementById('feedback-close').addEventListener('click', close_form, false);
    document.getElementById('feedback-overlay').addEventListener('click', may_close_form, false);

    document.getElementById('feedback-close-2').addEventListener('click', close_form, false);
    document.getElementById('feedback-overlay-2').addEventListener('click', close_form, false);

    document.getElementById('feedback-category').addEventListener(
        'change',
        function (e) {
            update_feedback_form(e.value);
        },
        false,
    );

    document.getElementById('feedback-send').addEventListener('click', send_form, false);

    if (feedback_preload) {
        open_form(feedback_preload.category, feedback_preload.subject, feedback_preload.body);
        update_feedback_form(feedback_preload.category);
    }

    return {
        open_form: open_form,
        close_form: close_form,
        update_feedback_form: update_feedback_form,
    };
})();
