setTimeout(() => {
  let failure = false
  function connect() {
    const socket = new WebSocket('/hc/ws')
    socket.onopen = () => {
      if (failure) {
        location.reload()
      }
    }
    socket.onclose = () => {
      failure = true
      setTimeout(connect, 500)
    }
  }
  connect()
}, 500)
