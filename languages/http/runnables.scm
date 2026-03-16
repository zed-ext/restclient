; Mark HTTP request method as runnable - creates ▶ Run button in gutter
; @run captures the method text (GET, POST, etc.)
; @url captures the target URL — becomes $ZED_CUSTOM_url for the task label/terminal title
(request
  method: (method) @run
  url: (target_url) @url
  (#set! tag http-request))
