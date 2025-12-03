#ifndef BARE_KIT_ANDROID_H
#define BARE_KIT_ANDROID_H

#include "bare-kit.h"

typedef struct ALooper ALooper;

struct bare_ipc_poll_s {
  bare_ipc_t *ipc;

  ALooper *looper;

  int events;

  bare_ipc_poll_cb cb;

  void *data;
};

ALooper *
ALooper_forThread();

void
ALooper_acquire(ALooper *looper);

void
ALooper_release(ALooper *looper);

#endif // BARE_KIT_ANDROID_H
