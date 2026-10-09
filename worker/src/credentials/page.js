(() => {
  const form = document.querySelector('form');
  if (!form) return;
  const input = document.getElementById('account-sk');
  const error = document.getElementById('field-error');
  const toggle = document.getElementById('visibility');
  const submit = document.getElementById('save');
  const cancel = document.getElementById('cancel');
  const title = document.getElementById('title');
  const description = document.getElementById('description');
  const expiry = Date.now() + Number(form.dataset.remainingMs);
  let busy = false;
  let expired = false;
  toggle.hidden = false;
  form.noValidate = true;
  const setError = (message) => {
    error.textContent = message;
    error.hidden = !message;
    input.setAttribute('aria-invalid', message ? 'true' : 'false');
  };
  const expire = () => {
    if (Date.now() < expiry || busy) return;
    expired = true;
    input.value = '';
    form.hidden = true;
    document.getElementById('privacy').hidden = true;
    title.textContent = '连接页面已过期';
    description.textContent = '请回到易界，重新打开 Sorftime 连接页面。';
    document.getElementById('status-note').hidden = false;
  };
  toggle.addEventListener('click', () => {
    const visible = input.type === 'password';
    input.type = visible ? 'text' : 'password';
    toggle.textContent = visible ? '隐藏' : '显示';
    toggle.setAttribute('aria-label', visible ? '隐藏 Account-SK' : '显示 Account-SK');
    toggle.setAttribute('aria-pressed', String(visible));
  });
  input.addEventListener('input', () => setError(''));
  form.addEventListener('submit', (event) => {
    expire();
    if (busy || expired) { event.preventDefault(); return; }
    // Cancel keeps its submitter value and bypasses required-field validation.
    if (event.submitter === cancel) { input.value = ''; return; }
    const token = input.value.replace(/^Bearer /, '');
    const message = !token ? '请粘贴 Account-SK。' :
      /\s/.test(token) || token.length > 4089 ? 'Account-SK 格式不正确，请重新复制完整密钥。' : '';
    if (message) { event.preventDefault(); setError(message); input.focus(); return; }
    busy = true;
    input.type = 'password';
    input.readOnly = true;
    submit.disabled = true;
    cancel.disabled = true;
    toggle.disabled = true;
    submit.textContent = '正在提交…';
    form.setAttribute('aria-busy', 'true');
  });
  window.addEventListener('pageshow', (event) => {
    // Never restore the key from a browser's back/forward snapshot.
    if (event.persisted) { input.value = ''; window.location.reload(); }
    expire();
  });
  document.addEventListener('visibilitychange', expire);
  setTimeout(expire, Math.max(0, expiry - Date.now()));
  expire();
})();
