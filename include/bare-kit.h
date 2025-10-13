#ifndef BARE_KIT_H
#define BARE_KIT_H

#ifdef __cplusplus
extern "C"
{
#endif

#include <bare.h>
#include <js.h>
#include <uv.h>
#include <stddef.h>

#define BARE_IPC_READ_BUFFER_SIZE 64 * 1024

  typedef struct bare_suspension_s bare_suspension_t;

#ifdef __ANDROID__
#include "android/suspension.h"
#endif

  int bare_suspension_init(bare_suspension_t *suspension);

  int bare_suspension_start(bare_suspension_t *suspension, int linger);

  int bare_suspension_end(bare_suspension_t *suspension);

  typedef struct bare_worklet_s bare_worklet_t;
  typedef struct bare_worklet_options_s bare_worklet_options_t;
  typedef struct bare_worklet_push_s bare_worklet_push_t;

  typedef void (*bare_worklet_finalize_cb)(bare_worklet_t *, const uv_buf_t *source, void *finalize_hint);
  typedef void (*bare_worklet_push_cb)(bare_worklet_push_t *, const char *error, const uv_buf_t *reply);

  struct bare_worklet_options_s
  {
    /**
     * The memory limit of each JavaScript heap. By default, the limit will be
     * inferred based on the amount of physical memory of the device.
     *
     * Note that the limit applies individually to each thread, including the
     * main thread.
     */
    size_t memory_limit;

    /**
     * The directory in which assets should be stored. The worklet require both
     * read and write access to the directory. By default, assets may not be
     * referenced and doing so will result in a runtime error.
     */
    const char *assets;
  };

  struct bare_worklet_s
  {
    bare_t *bare;

    bare_worklet_options_t options;

    bare_suspension_t suspension;

    const char *filename;

    struct
    {
      enum
      {
        bare_worklet_source_none = 0,
        bare_worklet_source_buffer = 1,
      } type;

      union
      {
        uv_buf_t buffer;
      };
    } source;

    bare_worklet_finalize_cb finalize;
    void *finalize_hint;

    int argc;
    const char **argv;

    uv_thread_t thread;
    uv_sem_t ready;

    uv_file incoming;
    uv_file outgoing;

    js_threadsafe_function_t *push;

    void *data;
  };

  struct bare_worklet_push_s
  {
    bare_worklet_t *worklet;

    bare_worklet_push_cb cb;

    uv_buf_t payload;

    void *data;
  };

  /**
   * Enable trade-off of performance for memory. Must be configured before
   * starting the first worklet.
   */
  int bare_worklet_optimize_for_memory(bool enabled);

  int bare_worklet_alloc(bare_worklet_t **result);

  int bare_worklet_init(bare_worklet_t *worklet, const bare_worklet_options_t *options);

  void
  bare_worklet_destroy(bare_worklet_t *worklet);

  void *
  bare_worklet_get_data(bare_worklet_t *worklet);

  void
  bare_worklet_set_data(bare_worklet_t *worklet, void *data);

  int bare_worklet_start(bare_worklet_t *worklet, const char *filename, const uv_buf_t *source, bare_worklet_finalize_cb finalize, void *finalize_hint, int argc, const char *argv[]);

  int bare_worklet_suspend(bare_worklet_t *worklet, int linger);

  int bare_worklet_resume(bare_worklet_t *worklet);

  int bare_worklet_terminate(bare_worklet_t *worklet);

  int bare_worklet_push(bare_worklet_t *worklet, bare_worklet_push_t *req, const uv_buf_t *payload, bare_worklet_push_cb cb);

  void *
  bare_worklet_push_get_data(bare_worklet_push_t *req);

  void
  bare_worklet_push_set_data(bare_worklet_push_t *req, void *data);

  typedef struct bare_ipc_s bare_ipc_t;
  typedef struct bare_ipc_poll_s bare_ipc_poll_t;

  typedef void (*bare_ipc_poll_cb)(bare_ipc_poll_t *, int events);

  struct bare_ipc_s
  {
    int incoming;
    int outgoing;
    char data[BARE_IPC_READ_BUFFER_SIZE];
  };

  enum
  {
    bare_ipc_readable = 0x1,
    bare_ipc_writable = 0x2,
  };

  enum
  {
    bare_ipc_would_block = -1,
    bare_ipc_error = -2,
  };

#ifdef __ANDROID__
#include "android/ipc.h"
#endif

  int bare_ipc_alloc(bare_ipc_t **result);

  int bare_ipc_init(bare_ipc_t *ipc, bare_worklet_t *worklet);

  void
  bare_ipc_destroy(bare_ipc_t *ipc);

  int bare_ipc_get_incoming(bare_ipc_t *ipc);

  int bare_ipc_get_outgoing(bare_ipc_t *ipc);

  int bare_ipc_read(bare_ipc_t *ipc, void **data, size_t *len);

  int bare_ipc_write(bare_ipc_t *ipc, const void *data, size_t len);

  int bare_ipc_poll_alloc(bare_ipc_poll_t **result);

  int bare_ipc_poll_init(bare_ipc_poll_t *poll, bare_ipc_t *ipc);

  void
  bare_ipc_poll_destroy(bare_ipc_poll_t *poll);

  void *
  bare_ipc_poll_get_data(bare_ipc_poll_t *poll);

  void
  bare_ipc_poll_set_data(bare_ipc_poll_t *poll, void *data);

  bare_ipc_t *
  bare_ipc_poll_get_ipc(bare_ipc_poll_t *poll);

  int bare_ipc_poll_start(bare_ipc_poll_t *poll, int events, bare_ipc_poll_cb cb);

  int bare_ipc_poll_stop(bare_ipc_poll_t *poll);

#ifdef __cplusplus
}
#endif

#endif // BARE_KIT_H
