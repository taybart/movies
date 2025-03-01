const query = { title_type: 'movie' }
async function submit() {
  console.log(query)
  const results = document.getElementById('results')
  results.innerHTML = ''
  const div = document.createElement('div')
  div.innerHTML = 'loading...'
  results.appendChild(div)
  const res = await fetch('/api', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(query),
  })
  const body = await res.json()
  if (res.status >= 400) {
    console.error(body)
    return
  }
  console.log(body)
  results.innerHTML = ''
  body.forEach((e) => {
    const a = document.createElement('a')
    // a.href = `/pages/${e.tconst}`
    a.href = `/api/item/${e.tconst}`
    const li = document.createElement('li')
    li.id = e.tconst
    li.innerHTML = `${e.title_type} ${e.start_year} ${e.primary_title}`
    a.appendChild(li)
    results.append(a)
  })
}

document.addEventListener(
  'DOMContentLoaded',
  () => {
    const search = document.getElementById('search')
    if (!search) {
      console.error('search not found')
      return
    }
    search.addEventListener(
      'input',
      debounce((ev) => {
        console.log('form', ev.target.id, ev.target.value)
        if (ev.target.id === 'title') {
          parseInputQuery(query, ev.target.value)
        } else {
          query[ev.target.id] = ev.target.value
        }
        if (query.title !== '') {
          // submit(ev)
        }
      }, 500),
    )
    search.addEventListener('submit', (ev) => {
      ev.preventDefault()
      console.log(ev.target)
      parseInputQuery(query, document.getElementById('title').value)
      // const title_type = document.getElementById('title_type').value
      // if (title_type !== '') {
      //   query.title_type = title_type
      // }
      if (query.title !== '') {
        submit(ev)
      }
    })
  },
  { once: true },
)
