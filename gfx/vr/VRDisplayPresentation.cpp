/* -*- Mode: C++; tab-width: 20; indent-tabs-mode: nil; c-basic-offset: 2 -*-
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "VRDisplayPresentation.h"

#include "mozilla/dom/DocGroup.h"
#include "mozilla/Unused.h"
#include "VRDisplayClient.h"
#include "VRLayerChild.h"

using namespace mozilla;
using namespace mozilla::gfx;

VRDisplayPresentation::VRDisplayPresentation(VRDisplayClient *aDisplayClient,
                                             const nsTArray<mozilla::dom::VRLayer>& aLayers,
                                             uint32_t aGroup)
  : mDisplayClient(aDisplayClient)
  , mDOMLayers(aLayers)
  , mGroup(aGroup)
{
  CreateLayers();
}

uint32_t
VRDisplayPresentation::GetGroup() const
{
  return mGroup;
}

void
VRDisplayPresentation::CreateLayers()
{
  if (mLayers.Length()) {
    return;
  }

  for (dom::VRLayer& layer : mDOMLayers) {
    dom::HTMLCanvasElement* canvasElement = layer.mSource;
    if (!canvasElement) {
      /// XXX In the future we will support WebVR in WebWorkers here
      continue;
    }

    Rect leftBounds(0.0, 0.0, 0.5, 1.0);
    if (layer.mLeftBounds.Length() == 4) {
      leftBounds.x = layer.mLeftBounds[0];
      leftBounds.y = layer.mLeftBounds[1];
      leftBounds.SetWidth(layer.mLeftBounds[2]);
      leftBounds.SetHeight(layer.mLeftBounds[3]);
    } else if (layer.mLeftBounds.Length() != 0) {
      /**
       * We ignore layers with an incorrect number of values.
       * In the future, VRDisplay.requestPresent may throw in
       * this case.  See https://github.com/w3c/webvr/issues/71
       */
      continue;
    }

    Rect rightBounds(0.5, 0.0, 0.5, 1.0);
    if (layer.mRightBounds.Length() == 4) {
      rightBounds.x = layer.mRightBounds[0];
      rightBounds.y = layer.mRightBounds[1];
      rightBounds.SetWidth(layer.mRightBounds[2]);
      rightBounds.SetHeight(layer.mRightBounds[3]);
    } else if (layer.mRightBounds.Length() != 0) {
      /**
       * We ignore layers with an incorrect number of values.
       * In the future, VRDisplay.requestPresent may throw in
       * this case.  See https://github.com/w3c/webvr/issues/71
       */
      continue;
    }

    VRManagerChild *manager = VRManagerChild::Get();
    if (!manager) {
      NS_WARNING("VRManagerChild::Get returned null!");
      continue;
    }

    nsCOMPtr<nsIEventTarget> target;
    nsIDocument* doc;
    doc = canvasElement->OwnerDoc();
    if (doc) {
      target = doc->EventTargetFor(TaskCategory::Other);
    }

    RefPtr<VRLayerChild> vrLayer =
      static_cast<VRLayerChild*>(manager->CreateVRLayer(mDisplayClient->GetDisplayInfo().GetDisplayID(),
                                                        leftBounds, rightBounds, target,
                                                        mGroup));
    if (!vrLayer) {
      NS_WARNING("CreateVRLayer returned null!");
      continue;
    }

    vrLayer->Initialize(canvasElement);

    mLayers.AppendElement(vrLayer);
  }
}

void
VRDisplayPresentation::DestroyLayers()
{
  for (VRLayerChild* layer : mLayers) {
    if (layer->IsIPCOpen()) {
      Unused << layer->SendDestroy();
    }
  }
  mLayers.Clear();
}

void
VRDisplayPresentation::GetDOMLayers(nsTArray<dom::VRLayer>& result)
{
  result = mDOMLayers;
}

VRDisplayPresentation::~VRDisplayPresentation()
{
  DestroyLayers();
  mDisplayClient->PresentationDestroyed();
}

void VRDisplayPresentation::SubmitFrame()
{
  for (VRLayerChild *layer : mLayers) {
    layer->SubmitFrame(mDisplayClient->GetDisplayInfo().GetFrameId());
    break; // Currently only one layer supported, submit only the first
  }
}
