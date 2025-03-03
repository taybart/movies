let failed = false
let reload = false
setTimeout(() => {
  setInterval(() => {
    const controller = new AbortController()
    const id = setTimeout(() => controller.abort(), 250)
    fetch('/hc', {
      signal: controller.signal,
    })
      .then((res) => {
        if (res.status === 200 && failed && !reload) {
          console.log('reload')
          reload = true
          window.location.reload()
        }
        clearTimeout(id)
      })
      .catch((e) => {
        console.log(e)
        if (!failed) {
          failed = true
          console.log('failed')
        }
      })
  }, 500)
}, 1000)
