const debounce = (callback, wait) => {
  let timeoutId = null
  return (...args) => {
    window.clearTimeout(timeoutId)
    timeoutId = window.setTimeout(() => {
      callback(...args)
    }, wait)
  }
}

function parseInputQuery(query, value) {
  const hasFilter = (i) => {
    return i.startsWith('year:') || i.startsWith('type:')
  }
  const items = value.split(' ').reduce(
    (res, i) => {
      res[hasFilter(i) ? 'filters' : 'title'].push(i)
      return res
    },
    {
      filters: [],
      title: [],
    },
  )
  items.filters.forEach((i) => {
    if (i.startsWith('year:')) {
      query.year = +i.substring(5)
    }
    if (i.startsWith('type:')) {
      query.title_type = i.substring(5)
    }
  })
  query.title = items.title.join(' ')
}
