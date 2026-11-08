# Alacritty terminal backend adaptation

Source: alacritty_terminal 0.26.0, upstream commit 94e7c8874e526b1e67b349d9ba30ddf81669119e.
Original Apache-2.0 license retained. Narrow Windows change: a borrowed job-object option creates the ConPTY child suspended, assigns it before any application instructions run, resumes its primary thread, and closes the primary-thread handle. This prevents fast descendants escaping ownership. Other terminal parsing behavior remains upstream.
