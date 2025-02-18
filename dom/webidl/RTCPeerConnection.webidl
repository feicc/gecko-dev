/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://w3c.github.io/webrtc-pc/#interface-definition
 */

callback RTCSessionDescriptionCallback = void (RTCSessionDescriptionInit description);
callback RTCPeerConnectionErrorCallback = void (DOMError error);
callback RTCStatsCallback = void (RTCStatsReport report);

enum RTCSignalingState {
    "stable",
    "have-local-offer",
    "have-remote-offer",
    "have-local-pranswer",
    "have-remote-pranswer",
    "closed"
};

enum RTCIceGatheringState {
    "new",
    "gathering",
    "complete"
};

enum RTCIceConnectionState {
    "new",
    "checking",
    "connected",
    "completed",
    "failed",
    "disconnected",
    "closed"
};

dictionary RTCDataChannelInit {
  boolean        ordered = true;
  unsigned short maxPacketLifeTime;
  unsigned short maxRetransmits;
  DOMString      protocol = "";
  boolean        negotiated = false;
  unsigned short id;

  // These are deprecated due to renaming in the spec, but still supported for Fx53
  unsigned short maxRetransmitTime;
};

dictionary RTCOfferAnswerOptions {
//  boolean voiceActivityDetection = true; // TODO: support this (Bug 1184712)
};

dictionary RTCAnswerOptions : RTCOfferAnswerOptions {
};

dictionary RTCOfferOptions : RTCOfferAnswerOptions {
  long    offerToReceiveVideo;
  long    offerToReceiveAudio;
  boolean iceRestart = false;
};

interface RTCDataChannel;

[Pref="media.peerconnection.enabled",
 JSImplementation="@mozilla.org/dom/peerconnection;1",
 Constructor (optional RTCConfiguration configuration,
              optional object? constraints)]
interface RTCPeerConnection : EventTarget  {
  [Throws, StaticClassOverride="mozilla::dom::RTCCertificate"]
  static Promise<RTCCertificate> generateCertificate (AlgorithmIdentifier keygenAlgorithm);

  [Pref="media.peerconnection.identity.enabled"]
  void setIdentityProvider (DOMString provider,
                            optional DOMString protocol,
                            optional DOMString username);
  [Pref="media.peerconnection.identity.enabled"]
  Promise<DOMString> getIdentityAssertion();
  Promise<RTCSessionDescriptionInit> createOffer (optional RTCOfferOptions options);
  Promise<RTCSessionDescriptionInit> createAnswer (optional RTCAnswerOptions options);
  Promise<void> setLocalDescription (RTCSessionDescriptionInit description);
  Promise<void> setRemoteDescription (RTCSessionDescriptionInit description);
  readonly attribute RTCSessionDescription? localDescription;
  readonly attribute RTCSessionDescription? currentLocalDescription;
  readonly attribute RTCSessionDescription? pendingLocalDescription;
  readonly attribute RTCSessionDescription? remoteDescription;
  readonly attribute RTCSessionDescription? currentRemoteDescription;
  readonly attribute RTCSessionDescription? pendingRemoteDescription;
  readonly attribute RTCSignalingState signalingState;
  Promise<void> addIceCandidate ((RTCIceCandidateInit or RTCIceCandidate)? candidate);
  readonly attribute boolean? canTrickleIceCandidates;
  readonly attribute RTCIceGatheringState iceGatheringState;
  readonly attribute RTCIceConnectionState iceConnectionState;
  [Pref="media.peerconnection.identity.enabled"]
  readonly attribute Promise<RTCIdentityAssertion> peerIdentity;
  [Pref="media.peerconnection.identity.enabled"]
  readonly attribute DOMString? idpLoginUrl;

  [ChromeOnly]
  attribute DOMString id;

  RTCConfiguration      getConfiguration ();
  [UnsafeInPrerendering, Deprecated="RTCPeerConnectionGetStreams"]
  sequence<MediaStream> getLocalStreams ();
  [UnsafeInPrerendering, Deprecated="RTCPeerConnectionGetStreams"]
  sequence<MediaStream> getRemoteStreams ();
  void addStream (MediaStream stream);

  // replaces addStream; fails if already added
  // because a track can be part of multiple streams, stream parameters
  // indicate which particular streams should be referenced in signaling

  RTCRtpSender addTrack(MediaStreamTrack track,
                        MediaStream stream,
                        MediaStream... moreStreams);
  void removeTrack(RTCRtpSender sender);

  sequence<RTCRtpSender> getSenders();
  sequence<RTCRtpReceiver> getReceivers();

  [ChromeOnly]
  void mozAddRIDExtension(RTCRtpReceiver receiver, unsigned short extensionId);
  [ChromeOnly]
  void mozAddRIDFilter(RTCRtpReceiver receiver, DOMString rid);

  void close ();
  attribute EventHandler onnegotiationneeded;
  attribute EventHandler onicecandidate;
  attribute EventHandler onsignalingstatechange;
  attribute EventHandler onaddstream; // obsolete
  attribute EventHandler onaddtrack;  // obsolete
  attribute EventHandler ontrack;     // replaces onaddtrack and onaddstream.
  attribute EventHandler onremovestream;
  attribute EventHandler oniceconnectionstatechange;
  attribute EventHandler onicegatheringstatechange;

  Promise<RTCStatsReport> getStats (optional MediaStreamTrack? selector);

  // Data channel.
  RTCDataChannel createDataChannel (DOMString label,
                                    optional RTCDataChannelInit dataChannelDict);
  attribute EventHandler ondatachannel;
};

// Legacy callback API

partial interface RTCPeerConnection {

  // Dummy Promise<void> return values avoid "WebIDL.WebIDLError: error:
  // We have overloads with both Promise and non-Promise return types"

  Promise<void> createOffer (RTCSessionDescriptionCallback successCallback,
                             RTCPeerConnectionErrorCallback failureCallback,
                             optional RTCOfferOptions options);
  Promise<void> createAnswer (RTCSessionDescriptionCallback successCallback,
                              RTCPeerConnectionErrorCallback failureCallback);
  Promise<void> setLocalDescription (RTCSessionDescriptionInit description,
                                     VoidFunction successCallback,
                                     RTCPeerConnectionErrorCallback failureCallback);
  Promise<void> setRemoteDescription (RTCSessionDescriptionInit description,
                                      VoidFunction successCallback,
                                      RTCPeerConnectionErrorCallback failureCallback);
  Promise<void> addIceCandidate (RTCIceCandidate candidate,
                                 VoidFunction successCallback,
                                 RTCPeerConnectionErrorCallback failureCallback);
  Promise<void> getStats (MediaStreamTrack? selector,
                          RTCStatsCallback successCallback,
                          RTCPeerConnectionErrorCallback failureCallback);
};
